use anyhow::{Error, Context, Result};
use c2::pb::c2_client::C2Client;
use c2::pb::{ReportTaskOutputRequest, TaskOutput, Task};

use chrono::{DateTime, NaiveTime, Utc};
use tonic::transport::Channel;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use tokio::time::Duration;
use tokio::task::JoinHandle;

pub struct AsyncTask {
    pub future_join_handle: JoinHandle<Result<(), Error>>,
    pub start_time: DateTime<Utc>,
    pub grpc_task: Task,
    pub print_reciever: Receiver<String>,
}

pub async fn handle_output_and_responses(
    start_time: NaiveTime,
    mut tavern_client: C2Client<Channel>,
    all_exec_futures_iter: IntoIter<i32, AsyncTask>,
    mut running_task_res_map: HashMap<i32, Vec<TaskOutput>>,
) -> Result<(HashMap<i32, AsyncTask>, HashMap<i32, Vec<TaskOutput>>)> {

    let mut running_exec_futures: HashMap<i32, AsyncTask> = HashMap::new();

    for exec_future in all_exec_futures_iter {
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

    Ok((running_exec_futures, running_task_res_map))
}
