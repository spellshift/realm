extern crate eldritch;
extern crate golem;

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use eldritch::pb::Tome;
use eldritch::{Output, Runtime};
use std::collections::HashMap;
use std::fs;
use std::process;
use tokio::task::JoinHandle;

mod inter;

struct ParsedTome {
    pub path: String,
    pub eldritch: String,
}

struct Handle {
    handle: JoinHandle<()>,
    path: String,
    output: Output,
}

async fn run_tomes(tomes: Vec<ParsedTome>) -> Result<Vec<String>> {
    let mut handles = Vec::new();
    for tome in tomes {
        let (runtime, output) = Runtime::new();
        let handle = tokio::task::spawn_blocking(move || {
            runtime.run(Tome {
                eldritch: tome.eldritch,
                parameters: HashMap::new(),
                file_names: Vec::new(),
            });
        });
        handles.push(Handle {
            handle,
            path: tome.path,
            output,
        });
    }

    let mut result = Vec::new();
    for handle in handles {
        match handle.handle.await {
            Ok(_) => {}
            Err(err) => {
                eprintln!(
                    "error waiting for tome to complete: {} {}",
                    handle.path, err
                );
                continue;
            }
        };
        let mut out = handle.output.collect();
        let errors = handle.output.collect_errors();
        if errors.len() > 0 {
            return Err(anyhow!("tome execution failed: {:?}", errors));
        }
        println!("OUTPUT: {:?}", out);
        result.append(&mut out);
    }

    Ok(result)
}

// async fn execute_tomes_in_parallel(
//     tome_name_and_content: Vec<(String, String)>,
// ) -> anyhow::Result<(i32, Vec<String>)> {
//     // Queue async tasks
//     let mut all_tome_futures: Vec<(String, _)> = vec![];
//     for tome_data in tome_name_and_content {
//         let (mut runtime, output) = Runtime::new();
//         runtime.with_stdout_reporting();

//         let tmp_row = (
//             tome_data.0.clone().to_string(),
//             tokio::task::spawn_blocking(move || {
//                 runtime.run(Tome {
//                     eldritch: tome_data.1,
//                     parameters: HashMap::new(),
//                     file_names: Vec::new(),
//                 });
//             }),
//         );
//         all_tome_futures.push(tmp_row)
//     }

//     let mut error_code = 0;
//     let mut result: Vec<String> = Vec::new();
//     for tome_task in all_tome_futures {
//         let tome_name: String = tome_task.0;
//         // Join our
//         let tome_result_thread_join: Result<()> = match tome_task.1.await {
//             Ok(local_thread_join_res) => Ok(()),
//             Err(_) => {
//                 error_code = 1;
//                 Err(anyhow::anyhow!("An error occured waiting for the tome thread to complete while executing {tome_name}."))
//             }
//         };

//         match tome_result_thread_join {
//             Ok(local_tome_result) => result.push(local_tome_result),
//             Err(task_error) => {
//                 error_code = 1;
//                 eprintln!("[TASK ERROR] {tome_name}: {task_error}");
//             }
//         }
//     }
//     Ok((error_code, result))
// }

fn main() -> anyhow::Result<()> {
    let matches = Command::new("golem")
        .arg(
            Arg::with_name("INPUT")
                .help("Set the tomes to run")
                .multiple_occurrences(true)
                .required(false),
        )
        .arg(
            Arg::with_name("interactive")
                .help("Run the interactive REPL")
                .short('i')
                .multiple_occurrences(false)
                .required(false),
        )
        .get_matches();

    if matches.contains_id("INPUT") {
        // Read Tomes
        let tome_files = matches.try_get_many::<String>("INPUT").unwrap().unwrap();
        let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(tome_path.clone())?;
            parsed_tomes.push(ParsedTome {
                path: tome_path,
                eldritch: tome_contents,
            });
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (error_code, result) = match runtime.block_on(run_tomes(parsed_tomes)) {
            Ok(response) => (0, response),
            Err(error) => {
                eprint!("failed to execute tome {:?}", error);
                (-1, Vec::new())
            }
        };

        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    } else if matches.contains_id("interactive") {
        inter::interactive_main()?;
    } else {
        let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
        for embedded_file_path in eldritch::assets::Asset::iter() {
            let filename = match embedded_file_path.split(r#"/"#).last() {
                Some(local_filename) => local_filename,
                None => "",
            };
            println!("{}", embedded_file_path);
            if filename == "main.eldritch" {
                let tome_path = embedded_file_path.to_string().clone();
                let tome_contents_extraction_result =
                    match eldritch::assets::Asset::get(embedded_file_path.as_ref()) {
                        Some(local_tome_content) => {
                            String::from_utf8(local_tome_content.data.to_vec())
                        }
                        None => {
                            eprint!("Failed to extract eldritch script as string");
                            Ok("".to_string())
                        }
                    };

                let tome_contents = match tome_contents_extraction_result {
                    Ok(local_tome_contents) => local_tome_contents,
                    Err(utf8_error) => {
                        eprint!("Failed to extract eldritch script as string {utf8_error}");
                        "".to_string()
                    }
                };
                parsed_tomes.push(ParsedTome {
                    path: tome_path,
                    eldritch: tome_contents,
                });
            }
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let (error_code, result) = match runtime.block_on(run_tomes(parsed_tomes)) {
            Ok(response) => (0, response),
            Err(error) => {
                eprint!("error executing tomes {:?}", error);
                (-1, Vec::new())
            }
        };

        if result.len() > 0 {
            println!("{:?}", result);
        }
        process::exit(error_code);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_golem_execute_tomes_in_parallel() -> anyhow::Result<()> {
        let parsed_tomes = Vec::from([ParsedTome {
            path: "test_hello.eldritch".to_string(),
            eldritch: r#"print("hello world")"#.to_string(),
        }]);

        let out = run_tomes(parsed_tomes).await?;
        assert_eq!("hello world".to_string(), out.join(""));
        Ok(())
    }
}
