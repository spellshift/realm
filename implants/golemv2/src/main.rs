mod agent;
mod assets;
mod assetbackend;

use clap::{Arg, Command, ArgAction};
use anyhow::Result;
use eldritchv2::{Interpreter, StdoutPrinter, assets};
use pb::eldritch::Tome;
use std::collections::HashMap;
use std::fs;
use std::process;
use walkdir::WalkDir;
use std::sync::Arc;
use rust_embed::RustEmbed;
use std::borrow::Cow;

// Get some embedded assets and implement them as AssetBackend
#[derive(RustEmbed)]
#[folder = "embedded"]
struct GolemEmbeddedAssets;
as_asset_backend!(GolemEmbeddedAssets);

struct ParsedTome {
    pub name: String,
    pub eldritch: String,
}

fn main() -> anyhow::Result<()>  {
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

    let mut assets: Vec<String> = Vec::new();
    let mut parsed_tomes: Vec<ParsedTome> = Vec::new();

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
                                parsed_tomes.push(ParsedTome {
                                    name: String::from(entry_str),
                                    eldritch: tome_contents
                                })
                            } else if entry.file_name() != "metadata.yml" {
                                assets.push(entry_str.to_owned());
                            }
                        }
                    }
                }
            }
        }
        // Set the assets of each tome to the local file names
        /*for tome in &mut parsed_tomes {
            tome.file_names = assets.clone();
        }
        */
    }

    if matches.contains_id("INPUT") {
        // Read Tomes from the input
        let tome_files = matches.try_get_many::<String>("INPUT").unwrap().unwrap();
        let mut parsed_tomes: Vec<ParsedTome> = Vec::new(); // Dont use the tomes in assets/ if we have specified some over the cmdline
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(&tome_path)?;
            parsed_tomes.push(ParsedTome {
                name: tome_path.clone(),
                eldritch: tome_contents,
            });
        }

        /*
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
        */
    } else if matches.contains_id("interactive") {
        eprint!("interactive is not implemented!\n");
        return Ok(());
    } else {
        // If we dont have any tomes (e.g. the user did not supply a local assets dir), then read the embedded tomes
        eprint!("we dont have any tomes specified\n");
    }

    let mut interp = Interpreter::new_with_printer(Arc::new(StdoutPrinter)).with_default_libs();

    // Register Task Context (Agent, Report, Assets)
    let remote_assets = assets.clone();
    let agent = agent::GolemAgent::new();
    interp = interp.with_task_context::<GolemEmbeddedAssets>(Arc::new(agent), 0, remote_assets);
    /*
    for tome in &mut parsed_tomes {
        println!("{}", tome.name);
    }
     */
    match interp.interpret("print(assets.list())\nprint(assets.read(assets.list()[1]))") {
        Ok(val) => {
            println!("{}", val)
        },
        Err(e) => {
            println!("{}", e)
        }
    }
    Ok(())
}
