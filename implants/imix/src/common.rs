use std::{time::{self, SystemTime, UNIX_EPOCH}, thread};

use crate::graphql::call;

use super::graphql;
use tokio::task::{self, JoinHandle};
use anyhow::{Result,Error};

enum TaskStatus {
    Waiting,
    Running,
    Finished,
}

struct TaskData {
    tome: String,
}

struct Task {
    task_id: String, // Task ID from tavern
    start_time: String, // When the task was started
    status: TaskStatus, // Wating, Running, Finished
    future_handle: JoinHandle<Result<(String), Error>>, // Handle to the task
    data: TaskData, //Not sure
}

async fn execute_tome(tome_path: String) {
    let ten_millis = time::Duration::from_secs(10);
    thread::sleep(ten_millis);    
}

// async fn graphql_request_update() -> graphql::GraphQLResponse {
//     unimplemented!("graphql will callback to the c2 server. Update the ")
// }

async fn pull_tome() {
    unimplemented!("graphql will callback to the c2 server. Update the ")
}

pub async fn main_loop() -> ! {
    let mut active_tasks: Vec<Task> = vec![];
    let mut graphql_responses: Vec<graphql::TaskResult> = vec![];
    loop {
        for task in &active_tasks {
            // Read off STDOUT 
            let task_output = "Some string".to_string();

            graphql_responses.push(graphql::TaskResult{
                task_id: task.task_id.clone(),
                exec_finished_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                output: task_output,
                error: "No errors. Never.".to_string(),
            })
        }

        match call("variables".to_string(), "uri".to_string(), 1234).await {
            Ok(respone) => {
                for new_task in [respone] {
                    active_tasks.append(Task{
                        task_id: new_task.,
                        start_time: todo!(),
                        status: todo!(),
                        future_handle: todo!(),
                        data: todo!(),
                    });
                }
            },
            Err(_) => todo!(),
        }
        // Get new tasks from the server.
        // let new_tasks = graphql_request_update();

        // let new_tasks = get_new_tasks();
        // for task in new_tasks {
        //     active_tasks.append()
        // }
    }
}