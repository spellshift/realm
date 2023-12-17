use anyhow::Result;
use clap::{Arg, Command};
use std::collections::HashMap;
use std::fs;
use std::process;
use std::thread;

use eldritch::{eldritch_run, StdPrintHandler};

async fn execute_tomes_in_parallel(
    tome_name_and_content: Vec<(String, String)>,
    custom_config: Option<&str>,
) -> anyhow::Result<(i32, Vec<String>)> {
    let tome_parameters = match custom_config {
        Some(config_path) => Some(HashMap::from([(
            "custom_config".to_string(),
            config_path.to_string(),
        )])),
        None => None,
    };

    // Queue async tasks
    let mut all_tome_futures: Vec<(String, _)> = vec![];
    for tome_data in tome_name_and_content {
        // let custom_config_string = custom_config.unwrap().to_string().to_owned();
        let local_tome_parameters = tome_parameters.clone();
        let tmp_row = (
            tome_data.0.clone().to_string(),
            thread::spawn(move || {
                eldritch_run(
                    tome_data.0,
                    tome_data.1,
                    local_tome_parameters,
                    &StdPrintHandler {},
                )
            }),
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
            }
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

pub fn install_main(custom_config: Option<&str>) -> anyhow::Result<()> {
    let mut tome_files_and_content: Vec<(String, String)> = Vec::new();
    for embedded_file_path in eldritch::assets::Asset::iter() {
        let filename = match embedded_file_path.split(r#"/"#).last() {
            Some(local_filename) => local_filename,
            None => "",
        };
        println!("{}", embedded_file_path);
        if filename == "main.eld" {
            let tome_path = embedded_file_path.to_string().clone();
            let tome_contents_extraction_result =
                match eldritch::assets::Asset::get(embedded_file_path.as_ref()) {
                    Some(local_tome_content) => String::from_utf8(local_tome_content.data.to_vec()),
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
            tome_files_and_content.push((tome_path, tome_contents))
        }
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (error_code, result) = match runtime.block_on(execute_tomes_in_parallel(
        tome_files_and_content,
        custom_config,
    )) {
        Ok(response) => response,
        Err(error) => {
            println!("Error executing tomes {:?}", error);
            (-1, Vec::new())
        }
    };

    if result.len() > 0 {
        println!("{:?}", result);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn imix_test_execute_tomes_in_parallel() -> anyhow::Result<()> {
        let tome_files_and_content = [("test_hello.eld".to_string(), "'hello world'".to_string())];
        let (error_code, result) =
            execute_tomes_in_parallel(tome_files_and_content.to_vec(), None).await?;
        assert_eq!(error_code, 0);
        assert!(result.contains(&"hello world".to_string()));
        Ok(())
    }
}
