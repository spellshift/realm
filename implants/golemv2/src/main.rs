#![allow(clippy::mutable_key_type)]
extern crate alloc;

use clap::{Arg, Command, ArgAction};
use eldritchv2::{ForeignValue, Interpreter, StdoutPrinter};
use std::fs;
use std::sync::Arc;
use std::borrow::Cow;
use eldritch_libassets::AssetsLibrary;
mod assetbackend;
mod multiassets;
mod agent;

use crate::agent::GolemAgent;
use crate::assetbackend::DirectoryAssetBackend;
use crate::multiassets::{MultiAssetLibrary, ParsedTome};

// Get some embedded assets and implement them as AssetBackend and RustEmbed
#[cfg(not(debug_assertions))]
asset_backend_embedded!(GolemEmbeddedAssets, "embedded");

#[cfg(debug_assertions)]
asset_backend_embedded!(GolemEmbeddedAssets, "../../bin/embedded_files_test");

// Build a new runtime
fn new_runtime(agent: Arc<GolemAgent>, assetlib: impl ForeignValue + 'static) -> Interpreter {
    // Maybe change the printer here?
    let mut interp = Interpreter::new_with_printer(Arc::new(StdoutPrinter)).with_default_libs();
    // Register the libraries that we need. Basically the same as interp.with_task_context but
    // with our custom assets library
    let agent_lib = eldritch_libagent::std::StdAgentLibrary::new(agent.clone(), 0);
    interp.register_lib(agent_lib);
    let report_lib = eldritch_libreport::std::StdReportLibrary::new(agent.clone(), 0);
    interp.register_lib(report_lib);
    let pivot_lib = eldritch_libpivot::std::StdPivotLibrary::new(agent.clone(), 0);
    interp.register_lib(pivot_lib);
    interp.register_lib(assetlib);
    interp
}

fn main() -> anyhow::Result<()>  {
    let matches = Command::new("golem")
        .arg(
            Arg::new("INPUT")
                .help("Set the tomes to run")
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("interactive")
                .help("Run the interactive REPL")
                .short('i')
                .required(false),
        )
        .arg(
            Arg::new("assets")
                .short('a')
                .long("assets")
                .value_name("SOURCE")
                .action(ArgAction::Append) 
                .help("Asset source (directory, Tavern URL, or ZIP)"),
        )
        .arg(
            Arg::new("embedded")
                .short('e')
                .long("embedded")
                .action(ArgAction::SetTrue)
                .help("Use embedded assets alongside asset sources"),
        )
        .arg(
            Arg::new("dump")
                .short('d')
                .long("dump")
                .action(ArgAction::SetTrue)
                .help("Dump tomes to be run and assets"),
        )
        .get_matches();

    //let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
    let mut locker = MultiAssetLibrary::new()?;

    let asset_directories: Vec<String> = matches.get_many::<String>("assets")
        .unwrap_or_default()
        .cloned()
        .collect();

    // If we have specified asset sources, we need to manually include embedded
    if asset_directories.len() == 0 || matches.get_flag("embedded") {
        locker.add(GolemEmbeddedAssets)?;
    }
    // Load all the asset directories into the locker
    for dir in asset_directories {
        let backend = DirectoryAssetBackend::new(&dir)?;
        locker.add(backend)?;
    }

    let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
    // Input overrides the main.eldritch files in the assets
    if matches.contains_id("INPUT") {
        // Read Tomes from the input
        let tome_files = matches.try_get_many::<String>("INPUT").unwrap().unwrap();
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            // Read the specified tome off of the disk first, if that fails. try to get it from the asset locker
            let tome_contents = fs::read_to_string(&tome_path)
                .map_err(|_| ())
                .or_else(|_| {
                    locker.read(tome_path.clone())
                    .map_err(|_| anyhow::anyhow!("Error: No such file or directory"))
                })?;

            parsed_tomes.push(ParsedTome {
                name: tome_path.clone(),
                eldritch: tome_contents,
            });
        }
    } else if matches.contains_id("interactive") {
        eprint!("interactive is not implemented!\n");
        return Ok(());
    }
    // If we havent specified tomes in INPUT, we need to look through the asset locker for tomes to run
    if parsed_tomes.len() == 0 {
        parsed_tomes = locker.tomes();
    }

    // Setup the interpreter. This will need refactored when we do multi-threaded
    let agent = Arc::new(GolemAgent::new());
    let mut interp = new_runtime(agent, locker);

    // Print a debug for the configured assets and tomes
    if matches.get_flag("dump") {
        let tome_names: Vec<&str> = parsed_tomes.iter()
            .map(|tome| tome.name.as_str())
            .collect();
        println!("tomes = {:?}", tome_names);
        match interp.interpret("print(\"assets =\", assets.list())") {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e)
            }
        }
    }

    // Time to run some commands
    for tome in parsed_tomes {
        match interp.interpret(&tome.eldritch) {
            Ok(_) => {},
            Err(e) => {
                println!("[{}] {}", tome.name, e)
            }
        }
    }

    Ok(())
}
