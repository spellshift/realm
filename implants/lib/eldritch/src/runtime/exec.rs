use super::{drain::drain, drain::drain_last, Environment, FileRequest};
use crate::{
    assets::AssetsLibrary,
    crypto::CryptoLibrary,
    file::FileLibrary,
    pb::{Credential, File, ProcessList, Tome},
    pivot::PivotLibrary,
    process::ProcessLibrary,
    report::ReportLibrary,
    sys::SysLibrary,
    time::TimeLibrary,
};
use anyhow::{Context, Error, Result};
use chrono::Utc;
use prost_types::Timestamp;
use starlark::{
    collections::SmallMap,
    environment::{Globals, GlobalsBuilder, LibraryExtension, Module},
    eval::Evaluator,
    starlark_module,
    syntax::{AstModule, Dialect},
    values::dict::Dict,
    values::AllocValue,
};
use std::sync::mpsc::{channel, Receiver};
use tokio::task::JoinHandle;

pub async fn start(tome: Tome) -> Runtime {
    let (tx_exec_started_at, rx_exec_started_at) = channel::<Timestamp>();
    let (tx_exec_finished_at, rx_exec_finished_at) = channel::<Timestamp>();
    let (tx_error, rx_error) = channel::<Error>();
    let (tx_output, rx_output) = channel::<String>();
    let (tx_credential, rx_credential) = channel::<Credential>();
    let (tx_process_list, rx_process_list) = channel::<ProcessList>();
    let (tx_file, rx_file) = channel::<File>();
    let (tx_file_request, rx_file_request) = channel::<FileRequest>();

    let env = Environment {
        tx_output,
        tx_error: tx_error.clone(),
        tx_credential,
        tx_process_list,
        tx_file,
        tx_file_request,
    };

    let handle = tokio::task::spawn_blocking(move || {
        // Send exec_started_at
        let start = Utc::now();
        match tx_exec_started_at.send(Timestamp {
            seconds: start.timestamp(),
            nanos: start.timestamp_subsec_nanos() as i32,
        }) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_started_at (tome={:?}): {}", tome, _err);
            }
        }

        #[cfg(debug_assertions)]
        log::info!("evaluating tome: {:?}", tome);

        // Run Tome
        match run_impl(env, &tome) {
            Ok(_) => {
                #[cfg(debug_assertions)]
                log::info!("tome evaluation successful (tome={:?})", tome);
            }
            Err(err) => {
                #[cfg(debug_assertions)]
                log::info!("tome evaluation failed (tome={:?}): {}", tome, err);

                // Report evaluation errors
                match tx_error.send(err) {
                    Ok(_) => {}
                    Err(_send_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to report tome evaluation error (tome={:?}): {}",
                            tome,
                            _send_err
                        );
                    }
                }
            }
        };

        // Send exec_finished_at
        let end = Utc::now();
        match tx_exec_finished_at.send(Timestamp {
            seconds: end.timestamp(),
            nanos: end.timestamp_subsec_nanos() as i32,
        }) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!(
                    "failed to send exec_finished_at (tome={:?}): {}",
                    tome,
                    _err
                );
            }
        }
    });

    Runtime {
        handle: Some(handle),
        rx_exec_started_at,
        rx_exec_finished_at,
        rx_error,
        rx_output,
        rx_credential,
        rx_process_list,
        rx_file,
        rx_file_request,
    }
}

fn run_impl(env: Environment, tome: &Tome) -> Result<()> {
    let ast = Runtime::parse(tome).context("failed to parse tome")?;
    let module = Runtime::alloc_module(tome).context("failed to allocate module")?;
    let globals = Runtime::globals();
    let mut eval: Evaluator = Evaluator::new(&module);
    eval.extra = Some(&env);
    eval.set_print_handler(&env);

    match eval.eval_module(ast, &globals) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into_anyhow().context("failed to evaluate tome")),
    }
}

/*
 * Eldritch Runtime
 *
 * This runtime is responsible for executing Tomes and reporting their output.
 * It acts as an interface between callers and starlark, exposing our standard libraries to the starlark interpreter.
 * It is also used to provide dependency injection for eldritch library functions (using `Runtime::from_extra(starlark_interpreter.extra)`).
 */
pub struct Runtime {
    handle: Option<JoinHandle<()>>,
    rx_exec_started_at: Receiver<Timestamp>,
    // stdout_reporting: bool,
    // exec_started_at: Receiver<Timestamp>,
    rx_exec_finished_at: Receiver<Timestamp>,
    rx_output: Receiver<String>,
    rx_error: Receiver<Error>,
    rx_credential: Receiver<Credential>,
    rx_process_list: Receiver<ProcessList>,
    rx_file: Receiver<File>,
    rx_file_request: Receiver<FileRequest>,
    // client: Client,
}

impl Runtime {
    /*
     * Globals available to eldritch code.
     * This provides all of our starlark standard libraries.
     */
    pub fn globals() -> Globals {
        #[starlark_module]
        fn eldritch(builder: &mut GlobalsBuilder) {
            const file: FileLibrary = FileLibrary;
            const process: ProcessLibrary = ProcessLibrary;
            const sys: SysLibrary = SysLibrary;
            const pivot: PivotLibrary = PivotLibrary;
            const assets: AssetsLibrary = AssetsLibrary;
            const crypto: CryptoLibrary = CryptoLibrary;
            const time: TimeLibrary = TimeLibrary;
            const report: ReportLibrary = ReportLibrary;
        }

        GlobalsBuilder::extended_by(&[
            LibraryExtension::StructType,
            LibraryExtension::RecordType,
            LibraryExtension::EnumType,
            LibraryExtension::Map,
            LibraryExtension::Filter,
            LibraryExtension::Partial,
            LibraryExtension::Debug,
            LibraryExtension::Print,
            LibraryExtension::Pprint,
            LibraryExtension::Breakpoint,
            LibraryExtension::Json,
            LibraryExtension::CallStack,
            LibraryExtension::Typing,
        ])
        .with(eldritch)
        .build()
    }

    /*
     * Parse an Eldritch tome into a starlark Abstract Syntax Tree (AST) Module.
     */
    fn parse(tome: &Tome) -> anyhow::Result<AstModule> {
        match AstModule::parse("main", tome.eldritch.to_string(), &Dialect::Extended) {
            Ok(v) => Ok(v),
            Err(err) => Err(err.into_anyhow()),
        }
    }

    /*
     * Allocate tome parameters on a new starlark module and return it, ready for execution.
     */
    fn alloc_module(tome: &Tome) -> Result<Module> {
        let module: Module = Module::new();
        let mut input_params: Dict = Dict::new(SmallMap::new());

        for (key, value) in &tome.parameters {
            let new_key = module.heap().alloc_str(key);
            let new_value = module.heap().alloc_str(value.as_str()).to_value();
            let hashed_key = match new_key.to_value().get_hashed() {
                Ok(local_hashed_key) => local_hashed_key,
                Err(local_error) => {
                    return Err(anyhow::anyhow!(
                        "[eldritch] Failed to create hashed key for key {}: {}",
                        new_key.to_string(),
                        local_error.to_string()
                    ))
                }
            };
            input_params.insert_hashed(hashed_key, new_value);
        }
        module.set("input_params", input_params.alloc_value(module.heap()));
        module.set(
            "remote_assets",
            tome.file_names.clone().alloc_value(module.heap()),
        );

        Ok(module)
    }

    /*
     * Returns true if the tome has completed execution, false otherwise.
     */
    pub fn is_finished(&self) -> bool {
        match &self.handle {
            Some(handle) => handle.is_finished(),
            None => true,
        }
    }

    /*
     * finish() yields until the tome has finished.
     */
    pub async fn finish(&mut self) {
        match self.handle.take() {
            Some(handle) => match handle.await {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to join runtime handle: {}", _err);
                }
            },
            None => {
                #[cfg(debug_assertions)]
                log::error!("attempted to join runtime handle which has already finished");
            }
        };
    }

    /*
     * Returns the timestamp of when execution started, if available.
     */
    pub fn get_exec_started_at(&self) -> Option<Timestamp> {
        drain_last(&self.rx_exec_started_at)
    }

    /*
     * Returns the timestamp of when execution finished, if available.
     */
    pub fn get_exec_finished_at(&self) -> Option<Timestamp> {
        drain_last(&self.rx_exec_finished_at)
    }

    /*
     * Collects all currently available reported text output.
     */
    pub fn collect_text(&self) -> Vec<String> {
        drain(&self.rx_output)
    }

    /*
     * Collects all currently available reported errors, if any.
     */
    pub fn collect_errors(&self) -> Vec<Error> {
        drain(&self.rx_error)
    }

    /*
     * Returns all currently available reported credentials, if any.
     */
    pub fn collect_credentials(&self) -> Vec<Credential> {
        drain(&self.rx_credential)
    }

    /*
     * Returns all currently available reported process lists, if any.
     */
    pub fn collect_process_lists(&self) -> Vec<ProcessList> {
        drain(&self.rx_process_list)
    }

    /*
     * Returns all currently available reported files, if any.
     */
    pub fn collect_files(&self) -> Vec<File> {
        drain(&self.rx_file)
    }

    /*
     * Returns all FileRequests that the eldritch runtime has requested, if any.
     */
    pub fn collect_file_requests(&self) -> Vec<FileRequest> {
        drain(&self.rx_file_request)
    }
}
