#![deny(warnings)]

extern crate eldritch;
extern crate golem;

mod inter;

use anyhow::{anyhow, Result};
use clap::{Arg, ArgAction, Command};
use eldritch::runtime::{messages::AsyncMessage, Message};
use pb::eldritch::Tome;
use std::collections::HashMap;
use std::fs;
use std::process;
use walkdir::WalkDir;

async fn run_tomes(tomes: Vec<Tome>) -> Result<Vec<String>> {
    let mut runtimes = Vec::new();
    let mut idx = 1;
    for tome in tomes {
        // Check if we have local files, if so, we set the file_names type to "local_files"
        let runtime = eldritch::start(idx, tome).await;
        runtimes.push(runtime);
        idx += 1;
    }

    let mut result = Vec::new();
    let mut errors = Vec::new();
    for runtime in &mut runtimes {
        runtime.finish().await;

        for msg in runtime.messages() {
            match msg {
                Message::Async(AsyncMessage::ReportText(m)) => result.push(m.text()),
                Message::Async(AsyncMessage::ReportError(m)) => errors.push(m.error),
                _ => {}
            }
        }
    }
    if !errors.is_empty() {
        return Err(anyhow!("{:?}", errors));
    }
    Ok(result)
}

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
        .arg(
            Arg::new("assets")
                .short('a')
                .long("assets")
                .action(ArgAction::Set)
                .value_name("DIRECTORY")
                .help("Local asset directory to expose to tomes"),
        )
        .get_matches();

    // Handle taking assets and tomes from the local file system instead of from the builtin
    let mut assets: Vec<String> = Vec::new();
    let mut parsed_tomes: Vec<Tome> = Vec::new();

    if matches.contains_id("assets") {
        const MAX_RECURSION_DEPTH: usize = 10;
        let asset_directories = matches.try_get_many::<String>("assets").unwrap().unwrap();
        for dir in asset_directories {
            for entry in WalkDir::new(dir)
                .max_depth(MAX_RECURSION_DEPTH)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() {
                    // Make the path relative
                    if let Ok(rel_path) = entry.path().strip_prefix(dir) {
                        if let Some(entry_str) = rel_path.to_str() {
                            // If we have a tome, then add it
                            if entry.file_name() == "main.eldritch" {
                                let tome_contents = fs::read_to_string(entry.path())?;
                                let mut parameters = HashMap::new();
                                parameters.insert(
                                    "__file_names_type".to_string(),
                                    "local_assets".to_string(),
                                );

                                // All the assets will be relative to this pass
                                parameters.insert(
                                    "__local_assets_directory".to_string(),
                                    dir.to_string(),
                                );
                                parsed_tomes.push(Tome {
                                    eldritch: tome_contents,
                                    file_names: Vec::new(), // Leave this blank for now, we will repopulate it after Walking
                                    parameters,
                                })
                            }
                            assets.push(entry_str.to_owned());
                        }
                    }
                }
            }
        }
        // Set the assets of each tome to the local file names
        for tome in &mut parsed_tomes {
            tome.file_names = assets.clone();
        }
    }

    if matches.contains_id("INPUT") {
        // Read Tomes
        let tome_files = matches.try_get_many::<String>("INPUT").unwrap().unwrap();
        let mut parsed_tomes: Vec<Tome> = Vec::new();
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(tome_path)?;
            parsed_tomes.push(Tome {
                eldritch: tome_contents,
                parameters: HashMap::new(),
                file_names: Vec::new(),
            });
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (error_code, _result) = match runtime.block_on(run_tomes(parsed_tomes)) {
            Ok(response) => (0, response),
            Err(error) => {
                eprint!("failed to execute tome {:?}", error);
                (-1, Vec::new())
            }
        };

        process::exit(error_code);
    } else if matches.contains_id("interactive") {
        inter::interactive_main()?;
    } else {
        // If we dont have any tomes (e.g. the user did not supply a local assets dir, then read the embedded)
        // TODO: Should this be local assets AND embedded assets. An OR is consistent with REMOTE_ASSETS behavior
        // I dont think so, if I give it a dir, i dont want anything else to run....
        if parsed_tomes.is_empty() {
            for embedded_file_path in eldritch::assets::Asset::iter() {
                let filename = embedded_file_path.split('/').last().unwrap_or("");
                println!("{}", embedded_file_path);
                if filename == "main.eldritch" {
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
                    parsed_tomes.push(Tome {
                        eldritch: tome_contents,
                        file_names: Vec::new(),
                        parameters: HashMap::new(),
                    });
                }
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

        if !result.is_empty() {
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
        let parsed_tomes = Vec::from([Tome {
            eldritch: r#"print("hello world")"#.to_string(),
            parameters: HashMap::new(),
            file_names: Vec::new(),
        }]);

        let out = run_tomes(parsed_tomes).await?;
        assert_eq!("hello world\n".to_string(), out.join(""));
        Ok(())
    }
}
