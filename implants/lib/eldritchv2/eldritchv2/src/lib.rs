#![cfg_attr(feature = "no_std", no_std)]
#![allow(unexpected_cfgs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Re-exports from eldritch-stdlib
pub use eldritch_libagent as agent;
pub use eldritch_libassets as assets;
pub use eldritch_libcrypto as crypto;
pub use eldritch_libfile as file;
pub use eldritch_libhttp as http;
pub use eldritch_libpivot as pivot;
pub use eldritch_libprocess as process;
pub use eldritch_librandom as random;
pub use eldritch_libregex as regex;
pub use eldritch_libreport as report;
pub use eldritch_libsys as sys;
pub use eldritch_libtime as time;

// Re-export core types
pub use eldritch_core::{
    BufferPrinter, Environment, ForeignValue, Interpreter as CoreInterpreter, Printer, Span,
    StdoutPrinter, TokenKind, Value, conversion,
};

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "stdlib")]
use crate::agent::{agent::Agent, std::StdAgentLibrary};
#[cfg(feature = "stdlib")]
use crate::assets::std::StdAssetsLibrary;
#[cfg(feature = "stdlib")]
use crate::crypto::std::StdCryptoLibrary;
#[cfg(feature = "stdlib")]
use crate::file::std::StdFileLibrary;
#[cfg(feature = "stdlib")]
use crate::http::std::StdHttpLibrary;
#[cfg(feature = "stdlib")]
use crate::pivot::std::StdPivotLibrary;
#[cfg(feature = "stdlib")]
use crate::process::std::StdProcessLibrary;
#[cfg(feature = "stdlib")]
use crate::random::std::StdRandomLibrary;
#[cfg(feature = "stdlib")]
use crate::regex::std::StdRegexLibrary;
#[cfg(feature = "stdlib")]
use crate::report::std::StdReportLibrary;
#[cfg(feature = "stdlib")]
use crate::sys::std::StdSysLibrary;
#[cfg(feature = "stdlib")]
use crate::time::std::StdTimeLibrary;

#[cfg(feature = "fake_bindings")]
use crate::agent::fake::AgentLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::assets::fake::FakeAssetsLibrary;
#[cfg(feature = "fake_bindings")]
use crate::crypto::fake::CryptoLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::file::fake::FileLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::http::fake::HttpLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::pivot::fake::PivotLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::process::fake::ProcessLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::random::fake::RandomLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::regex::fake::RegexLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::report::fake::ReportLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::sys::fake::SysLibraryFake;
#[cfg(feature = "fake_bindings")]
use crate::time::fake::TimeLibraryFake;

pub struct Interpreter {
    inner: CoreInterpreter,
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
        #[cfg(feature = "stdlib")]
        {
            self.inner.register_lib(StdCryptoLibrary);
            self.inner.register_lib(StdFileLibrary);
            self.inner.register_lib(StdHttpLibrary);
            self.inner.register_lib(StdPivotLibrary::default());
            self.inner.register_lib(StdProcessLibrary);
            self.inner.register_lib(StdRandomLibrary);
            self.inner.register_lib(StdRegexLibrary);
            self.inner.register_lib(StdSysLibrary);
            self.inner.register_lib(StdTimeLibrary);
        }

        #[cfg(feature = "fake_bindings")]
        {
            self.inner.register_lib(CryptoLibraryFake);
            self.inner.register_lib(FileLibraryFake::default());
            self.inner.register_lib(HttpLibraryFake);
            self.inner.register_lib(PivotLibraryFake);
            self.inner.register_lib(ProcessLibraryFake);
            self.inner.register_lib(RandomLibraryFake);
            self.inner.register_lib(RegexLibraryFake);
            self.inner.register_lib(SysLibraryFake);
            self.inner.register_lib(TimeLibraryFake);
        }

        self
    }

    #[cfg(feature = "stdlib")]
    pub fn with_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        // Agent library needs a task_id. For general usage (outside of imix tasks),
        // we can use 0 or a placeholder.
        let agent_lib = StdAgentLibrary::new(agent.clone(), 0, String::new());
        self.inner.register_lib(agent_lib);

        let report_lib = StdReportLibrary::new(agent.clone(), 0, String::new());
        self.inner.register_lib(report_lib);

        let pivot_lib = StdPivotLibrary::new(agent.clone(), 0, String::new());
        self.inner.register_lib(pivot_lib);

        // Assets library
        let assets_lib =
            StdAssetsLibrary::<crate::assets::std::EmptyAssets>::new(agent.clone(), Vec::new());
        self.inner.register_lib(assets_lib);

        self
    }

    #[cfg(feature = "fake_bindings")]
    pub fn with_fake_agent(mut self) -> Self {
        self.inner.register_lib(AgentLibraryFake);
        self.inner.register_lib(ReportLibraryFake);
        self.inner.register_lib(PivotLibraryFake);
        self.inner.register_lib(FakeAssetsLibrary);
        self
    }

    #[cfg(feature = "stdlib")]
    pub fn with_task_context<A: crate::assets::RustEmbed + Send + Sync + 'static>(
        mut self,
        agent: Arc<dyn Agent>,
        task_id: i64,
        jwt: String,
        remote_assets: Vec<String>,
    ) -> Self {
        let agent_lib = StdAgentLibrary::new(agent.clone(), task_id, jwt.clone());
        self.inner.register_lib(agent_lib);

        let report_lib = StdReportLibrary::new(agent.clone(), task_id, jwt.clone());
        self.inner.register_lib(report_lib);

        let pivot_lib = StdPivotLibrary::new(agent.clone(), task_id, jwt);
        self.inner.register_lib(pivot_lib);

        let assets_lib = StdAssetsLibrary::<A>::new(agent, remote_assets);
        self.inner.register_lib(assets_lib);

        self
    }

    pub fn with_printer(self, printer: Arc<dyn Printer + Send + Sync>) -> Self {
        self.inner.env.write().printer = printer;
        self
    }

    pub fn register_lib(&mut self, lib: impl ForeignValue + 'static) {
        self.inner.register_lib(lib);
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

#[cfg(all(test, feature = "fake_bindings"))]
mod bindings_test;

#[cfg(all(test, feature = "fake_bindings"))]
mod input_params_test;
