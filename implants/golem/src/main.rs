extern crate golem;
extern crate eldritch;

use clap::{Command, Arg};
use tokio::task;
use std::fs;
use std::process;

use eldritch::{eldritch_run};

mod inter;


async fn run(tome_path: String) -> anyhow::Result<String> {
    // Read a tome script
    let tome_contents = fs::read_to_string(tome_path.clone())?;
    // Execute a tome script
    eldritch_run(tome_path, tome_contents)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("golem")
        .arg(Arg::with_name("INPUT")
            .help("Set the tomes to run")
            .multiple_occurrences(true)
            .required(false)
        ).get_matches();

    if matches.contains_id("INPUT") {
        // Get list of files
        let tome_files = matches.try_get_many::<String>("INPUT")
            .unwrap()
            .unwrap();


        // Queue async tasks
        let mut all_tome_futures: Vec<(String, _)> = vec![];
        for tome in tome_files {
            let tome_execution_task = run(tome.to_string());
            let tmp_row = (tome.to_string(), task::spawn(tome_execution_task));
            all_tome_futures.push(tmp_row)
        }

        let mut error_code = 0;
        // Collect results and do error handling
        let mut result: Vec<String> = Vec::new();
        for tome_task in all_tome_futures {
            let tome_name: String = tome_task.0;
            match tome_task.1.await {
                Ok(res) => match res {
                        Ok(task_res) => result.push(task_res),
                        Err(task_err) => {
                            eprintln!("[TASK ERROR] {tome_name}: {task_err}");
                            error_code = 1;        
                        }
                    },
                Err(err) => {
                    eprintln!("[ERROR] {tome_name}: {err}");
                    error_code = 1;
                },
            }
        }
        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    } else {
        inter::interactive_main()?;
    }
    Ok(())
}


