use std::{time, thread};

use anyhow::Error;
use tokio::task::JoinHandle;

use super::graphql;

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
    future_handle: JoinHandle<Result<String, Error>>, // Handle to the task
    data: TaskData, //Not sure
}

async fn execute_tome(tome_path: String) -> String {
    let ten_millis = time::Duration::from_secs(10);
    thread::sleep(ten_millis);
    return "A String".to_string()
}

pub async fn main_loop() {
    graphql::gql_claim_tasks("http://127.0.0.1:80/graphql".to_string()).await;
    unimplemented!("Nothing here yet. ")
}