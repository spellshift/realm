use super::{Broker, Client, FileRequest};
use crate::pb::{File, ProcessList};
use crate::{
    assets::AssetsLibrary, crypto::CryptoLibrary, file::FileLibrary, pb::Tome, pivot::PivotLibrary,
    process::ProcessLibrary, sys::SysLibrary, time::TimeLibrary,
};
use anyhow::{Error, Result};
use chrono::Utc;
use prost_types::Timestamp;
use starlark::PrintHandler;
use starlark::{
    collections::SmallMap,
    environment::{Globals, GlobalsBuilder, LibraryExtension, Module},
    eval::Evaluator,
    starlark_module,
    syntax::{AstModule, Dialect},
    values::dict::Dict,
    values::AllocValue,
};
use std::sync::mpsc::{channel, Sender};

/*
 * Eldritch Runtime
 *
 * This runtime is responsible for executing Tomes and reporting their output.
 * It acts as an interface between callers and starlark, exposing our standard libraries to the starlark interpreter.
 * It is also used to provide dependency injection for eldritch library functions (using `Runtime::from_extra(starlark_interpreter.extra)`).
 */
pub struct Runtime {
    stdout_reporting: bool,

    ch_exec_started_at: Sender<Timestamp>,
    ch_exec_finished_at: Sender<Timestamp>,

    client: Client,
}

impl Runtime {
    /*
     * Prepare a new Runtime for execution of a single tome.
     */
    pub fn new() -> (Runtime, Broker) {
        let (ch_exec_started_at, exec_started_at) = channel::<Timestamp>();
        let (ch_exec_finished_at, exec_finished_at) = channel::<Timestamp>();
        let (ch_error, errors) = channel::<Error>();
        let (ch_output, outputs) = channel::<String>();
        let (ch_process_list, process_lists) = channel::<ProcessList>();
        let (ch_file, files) = channel::<File>();
        let (ch_file_requests, file_requests) = channel::<FileRequest>();
        (
            Runtime {
                stdout_reporting: false,
                ch_exec_started_at,
                ch_exec_finished_at,
                client: Client {
                    ch_output,
                    ch_error,
                    ch_process_list,
                    ch_file,
                    ch_file_requests,
                },
            },
            Broker {
                exec_started_at,
                exec_finished_at,
                outputs,
                errors,
                process_lists,
                files,
                file_requests,
            },
        )
    }

    /*
     * Run an Eldritch tome, returning an error if it fails.
     * Output from the tome is exposed via channels, see `reported_output`, `reported_process_list`, and `reported_files`.
     */
    pub fn run(&self, tome: Tome) {
        match self.report_exec_started_at() {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_started_at: {_err}");
            }
        }

        match self.run_impl(tome) {
            Ok(_) => {}
            Err(err) => match self.client.report_error(err) {
                Ok(_) => {}
                Err(_send_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to send error: {_send_err}");
                }
            },
        }

        match self.report_exec_finished_at() {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_finished_at: {_err}");
            }
        }
    }

    fn run_impl(&self, tome: Tome) -> Result<()> {
        let ast = Runtime::parse(&tome)?;
        let module = Runtime::alloc_module(&tome)?;
        let globals = Runtime::globals();

        let mut eval: Evaluator = Evaluator::new(&module);
        eval.extra = Some(&self.client);
        eval.set_print_handler(self);

        match eval.eval_module(ast, &globals) {
            Ok(_) => Ok(()),
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("tome execution failed: {_err}");
                Err(_err)
            }
        }
    }

    /*
     * Globals available to eldritch code.
     * This provides all of our starlark standard libraries.
     */
    pub fn globals() -> Globals {
        #[starlark_module]
        fn eldritch(builder: &mut GlobalsBuilder) {
            const file: FileLibrary = FileLibrary();
            const process: ProcessLibrary = ProcessLibrary();
            const sys: SysLibrary = SysLibrary();
            const pivot: PivotLibrary = PivotLibrary();
            const assets: AssetsLibrary = AssetsLibrary();
            const crypto: CryptoLibrary = CryptoLibrary();
            const time: TimeLibrary = TimeLibrary();
        }

        GlobalsBuilder::extended_by(&[
            LibraryExtension::StructType,
            LibraryExtension::RecordType,
            LibraryExtension::EnumType,
            LibraryExtension::Map,
            LibraryExtension::Filter,
            LibraryExtension::Partial,
            LibraryExtension::ExperimentalRegex,
            LibraryExtension::Debug,
            LibraryExtension::Print,
            LibraryExtension::Breakpoint,
            LibraryExtension::Json,
            LibraryExtension::Abs,
            LibraryExtension::Typing,
        ])
        .with(eldritch)
        .build()
    }

    /*
     * Parse an Eldritch tome into a starlark Abstract Syntax Tree (AST) Module.
     */
    fn parse(tome: &Tome) -> Result<AstModule> {
        match AstModule::parse("main", tome.eldritch.to_string(), &Dialect::Extended) {
            Ok(res) => Ok(res),
            Err(err) => Err(anyhow::anyhow!(
                "[eldritch] Unable to parse eldritch tome: {}: {}",
                err.to_string(),
                tome.eldritch.to_string(),
            )),
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

        Ok(module)
    }

    /*
     * Print execution results to stdout as they become available.
     */
    pub fn with_stdout_reporting(&mut self) -> &mut Self {
        self.stdout_reporting = true;
        self
    }

    /*
     * Send exec_started_at timestamp.
     */
    fn report_exec_started_at(&self) -> Result<()> {
        let now = Utc::now();
        self.ch_exec_started_at.send(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        })?;
        Ok(())
    }

    /*
     * Send exec_finished_at timestamp.
     */
    fn report_exec_finished_at(&self) -> Result<()> {
        let now = Utc::now();
        self.ch_exec_finished_at.send(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        })?;
        Ok(())
    }
}

/*
 * Enables Runtime to be used as a starlark print handler.
 */
impl PrintHandler for Runtime {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        self.client.report_output(text.to_string())?;
        if self.stdout_reporting {
            print!("{}", text);
        }
        Ok(())
    }
}
