extern crate golem;
extern crate eldritch;

use clap::{Command, Arg};
use tokio::task;
use std::fs;

use eldritch::{eldritch_run};

mod inter;


async fn run(tome_path: String) -> anyhow::Result<String> {
    // Read a tome script
    let tome_contents = fs::read_to_string(tome_path.clone())?;
    // Execute a tome script
    let tome_results: String = eldritch_run(tome_path, tome_contents).unwrap();
    // Return script output
    return Ok(tome_results)
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
        let res = matches.try_get_many::<String>("INPUT")
            .unwrap()
            .unwrap();


        // Queue async tasks
        let mut all_tome_futures: Vec<_> = vec![];
        for tome in res{
            let tome_execution_task = run(tome.to_string());
            all_tome_futures.push(task::spawn(tome_execution_task))
        }

        // Collect results
        let mut result: Vec<String> = Vec::new();
        for tome_task in all_tome_futures {
            match tome_task.await.unwrap() {
                Ok(res) => result.push(res),
                Err(_err) => continue,
            }
        }
        println!("{:?}", result);

    } else {
        inter::interactive_main()?;
    }
    Ok(())
}


