
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

mod assetbackend;
mod multiassets;
mod agent;

use crate::assetbackend::DirectoryAssetBackend;
use crate::multiassets::MultiAssetLibrary;

// Get some embedded assets and implement them as AssetBackend and RustEmbed
asset_backend_embedded!(GolemEmbeddedAssets, "embedded");

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

    //let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
    let mut locker = MultiAssetLibrary::new();

    locker.add(GolemEmbeddedAssets);
    // Get all the given asset dirs as a AssetBackend
    if matches.contains_id("assets") {
        let asset_directories = matches.try_get_many::<String>("assets").unwrap().unwrap();
        for dir in asset_directories {
            match DirectoryAssetBackend::new(dir) {
                Ok(ab) => locker.add(ab),
                Err(e) => eprintln!("failed to open assets: {}", e)
            }
        }
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
    }

    let mut interp = Interpreter::new_with_printer(Arc::new(StdoutPrinter)).with_default_libs();
    // Register the libraries that we need. Basically the same as interp.with_task_context but
    // with our custom assets library
    let agent = Arc::new(agent::GolemAgent::new());
    let agent_lib = eldritch_libagent::std::StdAgentLibrary::new(agent.clone(), 0);
    interp.register_lib(agent_lib);
    let report_lib = eldritch_libreport::std::StdReportLibrary::new(agent.clone(), 0);
    interp.register_lib(report_lib);
    let pivot_lib = eldritch_libpivot::std::StdPivotLibrary::new(agent.clone(), 0);
    interp.register_lib(pivot_lib);
    interp.register_lib(locker);

    match interp.interpret("print(assets.list())") {
        Ok(val) => {
            println!("{}", val)
        },
        Err(e) => {
            println!("{}", e)
        }
    }
    Ok(())
}
