#![allow(clippy::mutable_key_type)]
extern crate alloc;

use clap::{Arg, Command, ArgAction};
use eldritchv2::{ForeignValue, Interpreter, StdoutPrinter};
use std::fs;
use std::sync::Arc;
use eldritch_libagent::agent::{self, Agent};
use std::borrow::Cow;
use tokio::sync::Semaphore;
use tokio::task;
use futures::stream::{self, StreamExt};
use tokio;
mod assetbackend;
mod multiassets;
mod agent;



use crate::agent::GolemAgent;
use crate::assetbackend::DirectoryAssetBackend;
use crate::multiassets::{MultiAssetLibrary, ParsedTome};

const MAX_CONCURRENT_INTERPRETATIONS: usize = 16;

// Get some embedded assets and implement them as AssetBackend and RustEmbed
asset_backend_embedded!(GolemEmbeddedAssets, "embedded");

// Build a new runtime
fn new_runtime(agent: Arc<dyn Agent>, assetlib: impl ForeignValue + 'static) -> Interpreter {
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

async fn execute_all_tomes<A, F, T: Agent>(
    parsed_tomes: Vec<ParsedTome>, 
    agent: dyn<impl Agent>,
    assetlib: impl ForeignValue + 'static
) -> Vec<Result<String, String>>
where
    A: Agent + Clone + 'static,
    F: ForeignValue + Clone + 'static,
{
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_INTERPRETATIONS));
    
    // 1. Map: Create a stream of futures (tasks)
    let futures_stream = stream::iter(parsed_tomes).map(|tome| {
        let semaphore_clone = Arc::clone(&semaphore);
        // This is the future (an async block) that produces the final Result
        async move { 
            // ... (Acquire permit, spawn_blocking, handle results as before) ...
            
            let _permit = semaphore_clone.acquire_owned().await.expect("Semaphore failed");

            let interpret_result = task::spawn_blocking(move || {
                let interp = new_runtime(agent, locker); 
                interp.interpret(&tome.eldritch)
            }).await;

            match interpret_result {
                Ok(res) => res, 
                Err(e) => {
                    eprintln!("Error: Blocking task for tome '{}' failed to join: {:?}", tome.name, e);
                    Err(format!("Task failed to join: {:?}", e))
                }
            }
        }
    });
    return ;
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
    
    let mut parsed_tomes: Vec<ParsedTome> = Vec::new();
    // Input overrides the main.eldritch files in the assets
    if matches.contains_id("INPUT") {
        // Read Tomes from the input
        let tome_files = matches.try_get_many::<String>("INPUT").unwrap().unwrap();
        for tome in tome_files {
            let tome_path = tome.to_string().clone();
            let tome_contents = fs::read_to_string(&tome_path)?;
            parsed_tomes.push(ParsedTome {
                name: tome_path.clone(),
                eldritch: tome_contents,
            });
        }

        /*

        */
    } else if matches.contains_id("interactive") {
        eprint!("interactive is not implemented!\n");
        return Ok(());
    }


    let agent = Arc::new(GolemAgent::new());
    let interp = new_runtime(agent, locker);
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
