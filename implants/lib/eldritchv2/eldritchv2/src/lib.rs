#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use eldritch_core::{Interpreter as CoreInterpreter, Printer, Value};
use eldritch_stdlib::{
    crypto::std::StdCryptoLibrary,
    file::std::StdFileLibrary,
    http::std::StdHttpLibrary,
    pivot::std::StdPivotLibrary,
    process::std::StdProcessLibrary,
    random::std::StdRandomLibrary,
    regex::std::StdRegexLibrary,
    sys::std::StdSysLibrary,
    time::std::StdTimeLibrary,
};
use eldritch_libagent::{agent::Agent, std::StdAgentLibrary};
use eldritch_libreport::std::StdReportLibrary;
use eldritch_libassets::std::StdAssetsLibrary;

pub struct Interpreter {
    pub inner: CoreInterpreter,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            inner: CoreInterpreter::new(),
        }
    }

    pub fn new_with_printer(printer: Arc<dyn Printer + Send + Sync>) -> Self {
        Self {
            inner: CoreInterpreter::new_with_printer(printer),
        }
    }

    pub fn with_default_libs(mut self) -> Self {
        self.register_module("crypto", Value::Foreign(Arc::new(StdCryptoLibrary)));
        self.register_module("file", Value::Foreign(Arc::new(StdFileLibrary)));
        self.register_module("http", Value::Foreign(Arc::new(StdHttpLibrary)));
        // StdPivotLibrary is a unit struct that delegates implementation.
        // It doesn't hold state (like an Agent), so it's safe to register as a default lib.
        self.register_module("pivot", Value::Foreign(Arc::new(StdPivotLibrary)));
        self.register_module("process", Value::Foreign(Arc::new(StdProcessLibrary)));
        self.register_module("random", Value::Foreign(Arc::new(StdRandomLibrary)));
        self.register_module("regex", Value::Foreign(Arc::new(StdRegexLibrary)));
        self.register_module("sys", Value::Foreign(Arc::new(StdSysLibrary)));
        self.register_module("time", Value::Foreign(Arc::new(StdTimeLibrary)));
        self
    }

    pub fn with_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        // Agent library needs a task_id. For general usage (outside of imix tasks),
        // we can use 0 or a placeholder.
        let agent_lib = StdAgentLibrary::new(agent.clone(), 0);
        self.register_module("agent", Value::Foreign(Arc::new(agent_lib)));

        let report_lib = StdReportLibrary::new(agent.clone(), 0);
        self.register_module("report", Value::Foreign(Arc::new(report_lib)));

        // Assets library
        let assets_lib = StdAssetsLibrary::new(agent.clone(), Vec::new());
        self.register_module("assets", Value::Foreign(Arc::new(assets_lib)));

        self
    }

    pub fn with_printer(mut self, printer: Arc<dyn Printer + Send + Sync>) -> Self {
        self.inner.env.write().printer = printer;
        self
    }

    // Proxy methods to inner interpreter

    pub fn interpret(&mut self, input: &str) -> Result<Value, String> {
        self.inner.interpret(input)
    }

    pub fn define_variable(&mut self, name: &str, value: Value) {
        self.inner.define_variable(name, value);
    }

    pub fn register_module(&mut self, name: &str, module: Value) {
        self.inner.register_module(name, module);
    }

    pub fn complete(&self, code: &str, cursor: usize) -> (usize, Vec<String>) {
        self.inner.complete(code, cursor)
    }
}
