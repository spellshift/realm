extern crate golem;
extern crate eldritch;

use clap::{Command, Arg};
use rust_embed::RustEmbed;
use std::fs;
use std::process;
use std::thread;

use eldritch::{eldritch_run};

mod inter;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

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
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(tome_path.clone())?;
            let tmp_row = (tome.to_string(), thread::spawn(|| { eldritch_run(tome_path, tome_contents, None) }));
            all_tome_futures.push(tmp_row)
        }


        let mut error_code = 0;
        let mut result: Vec<String> = Vec::new();
        for tome_task in all_tome_futures {
            let tome_name: String = tome_task.0;
            // Join our 
            let tome_result_thread_join = match tome_task.1.join() {
                Ok(local_thread_join_res) => local_thread_join_res,
                Err(_) => {
                    error_code = 1;
                    Err(anyhow::anyhow!("An error occured waiting for the tome thread to complete while executing {tome_name}."))
                },
            };

            match tome_result_thread_join {
                Ok(local_tome_result) => result.push(local_tome_result),
                Err(task_error) => {
                    error_code = 1;
                    eprintln!("[TASK ERROR] {tome_name}: {task_error}");
                }
            }
        }
        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    } else {
        for file in Asset::iter() {
            println!("{}", file.as_ref());
        }      
        // inter::interactive_main()?;
    }
    Ok(())
}
