use super::{drain::drain, drain::drain_last, Environment, FileRequest};
use crate::{
    assets::AssetsLibrary,
    crypto::CryptoLibrary,
    file::FileLibrary,
    pb::{Credential, File, ProcessList, Tome},
    pivot::PivotLibrary,
    process::ProcessLibrary,
    report::ReportLibrary,
    runtime::messages,
    runtime::Message,
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

pub async fn start(id: i64, tome: Tome) -> Runtime {
    let (tx, rx) = channel::<Message>();

    let env = Environment { id, tx };

    let handle = tokio::task::spawn_blocking(move || {
        // Send exec_started_at
        let start = Utc::now();
        match env.send(Message::ReportText(messages::ReportText {
            id,
            text: String::from(""),
            exec_started_at: Some(Timestamp {
                seconds: start.timestamp(),
                nanos: start.timestamp_subsec_nanos() as i32,
            }),
            exec_finished_at: None,
        })) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!(
                    "failed to send exec_started_at (task_id={}): {}",
                    env.id(),
                    _err
                );
            }
        }

        #[cfg(debug_assertions)]
        log::info!("evaluating tome (task_id={})", id);

        // Run Tome
        match run_impl(&env, &tome) {
            Ok(_) => {
                #[cfg(debug_assertions)]
                log::info!("tome evaluation successful (task_id={})", id);
            }
            Err(err) => {
                #[cfg(debug_assertions)]
                log::info!(
                    "tome evaluation failed (task_id={},tome={:?}): {}",
                    id,
                    tome,
                    err
                );

                // Report evaluation errors
                match env.send(Message::ReportError(messages::ReportError {
                    id,
                    error: err,
                    exec_started_at: None,
                    exec_finished_at: None,
                })) {
                    Ok(_) => {}
                    Err(_send_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to report tome evaluation error (task_id={},tome={:?}): {}",
                            id,
                            tome,
                            _send_err
                        );
                    }
                }
            }
        };

        // Send exec_finished_at
        let finish = Utc::now();
        match env.send(Message::ReportText(messages::ReportText {
            id,
            text: String::from(""),
            exec_started_at: None,
            exec_finished_at: Some(Timestamp {
                seconds: finish.timestamp(),
                nanos: finish.timestamp_subsec_nanos() as i32,
            }),
        })) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_started_at (task_id={}): {}", id, _err);
            }
        }
    });

    Runtime {
        handle: Some(handle),
        rx,
    }
}

fn run_impl(env: &Environment, tome: &Tome) -> Result<()> {
    let ast = Runtime::parse(tome).context("failed to parse tome")?;
    let module = Runtime::alloc_module(tome).context("failed to allocate module")?;
    let globals = Runtime::globals();
    let mut eval: Evaluator = Evaluator::new(&module);
    eval.extra = Some(env);
    eval.set_print_handler(env);

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
    rx: Receiver<Message>,
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
}
