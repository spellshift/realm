#![allow(clippy::mutable_key_type)]
extern crate alloc;

use clap::{Arg, ArgAction, Command};
use eldritchv2::agent::{fake::AgentFake, std::StdAgentLibrary};
use eldritchv2::assets::{
    AssetsLibrary,
    std::{EmbeddedAssets, StdAssetsLibrary},
};
use eldritchv2::conversion::ToValue;
use eldritchv2::{ForeignValue, Interpreter, StdoutPrinter};
use std::collections::BTreeMap;
use std::fs;
use std::process::exit;
use std::sync::Arc;

mod directorybackend;
mod repl;
use crate::directorybackend::DirectoryAssetBackend;

// Get some embedded assets and implement them as AssetBackend and RustEmbed
#[cfg(not(debug_assertions))]
#[derive(Debug, rust_embed::RustEmbed)]
#[folder = "embedded"]
pub struct GolemEmbeddedAssets;

#[cfg(debug_assertions)]
#[derive(Debug, rust_embed::RustEmbed)]
#[folder = "../../bin/embedded_files_test"]
pub struct GolemEmbeddedAssets;

pub struct ParsedTome {
    pub name: String,
    pub eldritch: String,
}

// Build a new runtime
fn new_runtime(assetlib: impl ForeignValue + 'static) -> Interpreter {
    // Maybe change the printer here?
    let mut interp = Interpreter::new_with_printer(Arc::new(StdoutPrinter)).with_default_libs();
    // Register the libraries that we need. Basically the same as interp.with_task_context but
    // with our custom assets library
    let agent = Arc::new(AgentFake {});
    let agent_lib = StdAgentLibrary::new(agent.clone(), 0);
    interp.register_lib(agent_lib);
    let report_lib = eldritchv2::report::std::StdReportLibrary::new(agent.clone(), 0);
    interp.register_lib(report_lib);
    let pivot_lib = eldritchv2::pivot::std::StdPivotLibrary::new(agent.clone(), 0);
    interp.register_lib(pivot_lib);
    interp.register_lib(assetlib);
    interp
}

fn main() -> anyhow::Result<()> {
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

    let mut locker = StdAssetsLibrary::new();

    let asset_directories: Vec<String> = matches
        .get_many::<String>("assets")
        .unwrap_or_default()
        .cloned()
        .collect();

    // If we have specified asset sources, we need to manually include embedded
    if asset_directories.is_empty() || matches.get_flag("embedded") {
        let backend = EmbeddedAssets::<GolemEmbeddedAssets>::new();
        locker.add(Arc::new(backend))?;
    }
    // Load all the asset directories into the locker
    for dir in asset_directories {
        let backend = DirectoryAssetBackend::new(&dir)?;
        locker.add(Arc::new(backend))?;
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
                    locker
                        .read(tome_path.clone())
                        .map_err(|_| anyhow::anyhow!("Error: No such file or asset"))
                })?;

            parsed_tomes.push(ParsedTome {
                name: tome_path.clone(),
                eldritch: tome_contents,
            });
        }
    }
    // If we havent specified tomes in INPUT, we need to look through the asset locker for tomes to run
    if parsed_tomes.is_empty() {
        match locker.list() {
            Ok(assets) => {
                for asset in assets {
                    if asset.ends_with("main.eldritch") {
                        let eldr_str = match locker.read(asset.clone()) {
                            Ok(val) => val,
                            Err(e) => return Err(anyhow::anyhow!(e)),
                        };
                        parsed_tomes.push(ParsedTome {
                            name: asset,
                            eldritch: eldr_str,
                        });
                    }
                }
            }
            Err(e) => return Err(anyhow::anyhow!(e)),
        }
    }

    // Setup the interpreter. This will need refactored when we do multi-threaded
    let mut interp = new_runtime(locker);

    if matches.contains_id("interactive") {
        repl::repl(interp)?;
        return Ok(());
    }

    // Print a debug for the configured assets and tomes
    if matches.get_flag("dump") {
        let tome_names: Vec<&str> = parsed_tomes.iter().map(|tome| tome.name.as_str()).collect();
        println!("tomes = {:?}", tome_names);
        match interp.interpret("print(\"assets =\", assets.list())") {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                exit(127);
            }
        }
    }

    // Time to run some commands
    for tome in parsed_tomes {
        // In the future we would like to set input params here.
        // We could compile them in for the default dropper assets
        let params: BTreeMap<String, String> = BTreeMap::new();
        interp.define_variable("input_params", params.to_value());
        match interp.interpret(&tome.eldritch) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}: {}", tome.name, e);
                exit(127);
            }
        }
    }

    Ok(())
}
