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
#[folder = "../../tests/embedded_files_test"]
pub struct Asset;


async fn execute_tomes_in_parallel(tome_name_and_content: Vec<(String, String)>) -> anyhow::Result<(i32, Vec<String>)> {
    // Queue async tasks
    let mut all_tome_futures: Vec<(String, _)> = vec![];
    for tome_data in tome_name_and_content {
        let tmp_row = (
            tome_data.0.clone().to_string(), 
            thread::spawn(|| { eldritch_run(tome_data.0, tome_data.1, None) })
        );
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
    Ok((error_code, result))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("golem")
        .arg(Arg::with_name("INPUT")
            .help("Set the tomes to run")
            .multiple_occurrences(true)
            .required(false)
        )
        .arg(Arg::with_name("interactive")
            .help("Run the interactive REPL")
            .short('i')
            .multiple_occurrences(false)
            .required(false)
        ).get_matches();

    if matches.contains_id("INPUT") {
        // Get list of files
        let tome_files = matches.try_get_many::<String>("INPUT")
            .unwrap()
            .unwrap();

        let mut tome_files_and_content: Vec< (String, String) > = Vec::new();
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(tome_path.clone())?;
            tome_files_and_content.push( (tome_path, tome_contents) )
        }

        let (error_code, result) = execute_tomes_in_parallel(tome_files_and_content).await?;
        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    } else if matches.contains_id("interactive") {
        inter::interactive_main()?;
    } else {
        let mut tome_files_and_content: Vec< (String, String) > = Vec::new();
        for embedded_file_path in Asset::iter() {
            let filename = match embedded_file_path.split(r#"/"#).last() {
                Some(local_filename) => local_filename,
                None => "",
            };
            println!("{}", embedded_file_path);
            if filename == "main.eld" {
                let tome_path = embedded_file_path.to_string().clone();
                let tome_contents_extraction_result = match Asset::get(embedded_file_path.as_ref()) {
                    Some(local_tome_content) => String::from_utf8(local_tome_content.data.to_vec()),
                    None => {
                        eprint!("Failed to extract eldritch script as string");
                        Ok("".to_string())
                    },
                };
                
                let tome_contents = match tome_contents_extraction_result {
                    Ok(local_tome_contents) => local_tome_contents,
                    Err(utf8_error) => {
                        eprint!("Failed to extract eldritch script as string {utf8_error}");
                        "".to_string()
                    },
                };
                tome_files_and_content.push( (tome_path, tome_contents) )
            }
    
        }      
        let (error_code, result) = execute_tomes_in_parallel(tome_files_and_content).await?;
        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    }

    Ok(())
}
