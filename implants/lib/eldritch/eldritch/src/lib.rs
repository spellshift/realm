#![cfg_attr(feature = "no_std", no_std)]
#![allow(unexpected_cfgs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Re-exports from eldritch-stdlib
pub use eldritch_libaes as aes;
pub use eldritch_libagent as agent;
pub use eldritch_libassets as assets;
pub use eldritch_libbase64 as base64;
pub use eldritch_libchain as chain;
pub use eldritch_libcrypto as crypto;
pub use eldritch_libdns as dns;
pub use eldritch_libfile as file;
pub use eldritch_libgzip as gzip;
pub use eldritch_libhash as hash;
pub use eldritch_libhttp as http;
pub use eldritch_libjson as json;
pub use eldritch_liblinux as linux;
pub use eldritch_libnet as net;
pub use eldritch_libpivot as pivot;
pub use eldritch_libportal as portal;
pub use eldritch_libprocess as process;
pub use eldritch_librandom as random;
pub use eldritch_libregex as regex;
pub use eldritch_libreport as report;
pub use eldritch_libscan as scan;
pub use eldritch_libssh as ssh;
pub use eldritch_libsys as sys;
pub use eldritch_libtemplate as template;
pub use eldritch_libtime as time;
pub use eldritch_libuser as user;
pub use eldritch_libutf8 as utf8;
pub use eldritch_libutf16le as utf16le;
pub use eldritch_libwindows as windows;
pub use eldritch_repl as repl;

// Re-export core types
pub use eldritch_core::{
    BufferPrinter, Environment, ForeignValue, Interpreter as CoreInterpreter, NoopPrinter, Printer,
    Span, StdoutPrinter, TokenKind, Value, conversion, format_tprint, pretty_format,
};
pub use eldritch_macros as macros;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "stdlib")]
use eldritch_agent::Context;

#[cfg(feature = "stdlib")]
use crate::aes::std::StdAesLibrary;
#[cfg(feature = "stdlib")]
pub use crate::agent::{agent::Agent, std::StdAgentLibrary};
#[cfg(feature = "stdlib")]
pub use crate::assets::std::AgentAssets;
#[cfg(feature = "stdlib")]
pub use crate::assets::std::AssetBackend;
#[cfg(feature = "stdlib")]
pub use crate::assets::std::EmbeddedAssets;
#[cfg(feature = "stdlib")]
pub use crate::assets::std::EmptyAssets;
#[cfg(feature = "stdlib")]
use crate::assets::std::StdAssetsLibrary;
#[cfg(feature = "stdlib")]
use crate::base64::std::StdBase64Library;
#[cfg(feature = "stdlib")]
use crate::chain::std::StdChainLibrary;
#[cfg(feature = "stdlib")]
use crate::crypto::std::StdCryptoLibrary;
#[cfg(feature = "stdlib")]
use crate::dns::std::StdDnsLibrary;
#[cfg(feature = "stdlib")]
use crate::file::std::StdFileLibrary;
#[cfg(feature = "stdlib")]
use crate::gzip::std::StdGzipLibrary;
#[cfg(feature = "stdlib")]
use crate::hash::std::StdHashLibrary;
#[cfg(feature = "stdlib")]
use crate::http::std::StdHttpLibrary;
#[cfg(feature = "stdlib")]
use crate::json::std::StdJsonLibrary;
#[cfg(feature = "stdlib")]
use crate::linux::std::StdLinuxLibrary;
#[cfg(feature = "stdlib")]
use crate::net::std::StdNetLibrary;
#[cfg(feature = "stdlib")]
use crate::pivot::std::StdPivotLibrary;
#[cfg(feature = "stdlib")]
use crate::portal::std::StdPortalLibrary;
#[cfg(feature = "stdlib")]
use crate::process::std::StdProcessLibrary;
#[cfg(feature = "stdlib")]
use crate::random::std::StdRandomLibrary;
#[cfg(feature = "stdlib")]
use crate::regex::std::StdRegexLibrary;
#[cfg(feature = "stdlib")]
use crate::report::std::StdReportLibrary;
#[cfg(feature = "stdlib")]
use crate::scan::std::StdScanLibrary;
#[cfg(feature = "stdlib")]
use crate::ssh::std::StdSshLibrary;
#[cfg(feature = "stdlib")]
use crate::sys::std::StdSysLibrary;
#[cfg(feature = "stdlib")]
use crate::template::std::StdTemplateLibrary;
#[cfg(feature = "stdlib")]
use crate::time::std::StdTimeLibrary;
#[cfg(feature = "stdlib")]
use crate::user::std::StdUserLibrary;
#[cfg(feature = "stdlib")]
use crate::utf8::std::StdUtf8Library;
#[cfg(feature = "stdlib")]
use crate::utf16le::std::StdUtf16leLibrary;
#[cfg(feature = "stdlib")]
use crate::windows::std::StdWindowsLibrary;

#[cfg(feature = "fake_aes")]
use crate::aes::fake::AesLibraryFake;
#[cfg(feature = "fake_agent")]
use crate::agent::fake::AgentLibraryFake;
#[cfg(feature = "fake_assets")]
use crate::assets::fake::FakeAssetsLibrary;
#[cfg(feature = "fake_base64")]
use crate::base64::fake::Base64LibraryFake;
#[cfg(feature = "fake_chain")]
use crate::chain::fake::FakeChainLibrary;
#[cfg(feature = "fake_crypto")]
use crate::crypto::fake::CryptoLibraryFake;
#[cfg(feature = "fake_dns")]
use crate::dns::fake::DnsLibraryFake;
#[cfg(feature = "fake_file")]
use crate::file::fake::FileLibraryFake;
#[cfg(feature = "fake_gzip")]
use crate::gzip::fake::GzipLibraryFake;
#[cfg(feature = "fake_hash")]
use crate::hash::fake::HashLibraryFake;
#[cfg(feature = "fake_http")]
use crate::http::fake::HttpLibraryFake;
#[cfg(feature = "fake_json")]
use crate::json::fake::JsonLibraryFake;
#[cfg(feature = "fake_linux")]
use crate::linux::fake::LinuxLibraryFake;
#[cfg(feature = "fake_net")]
use crate::net::fake::NetLibraryFake;
#[cfg(feature = "fake_pivot")]
use crate::pivot::fake::PivotLibraryFake;
#[cfg(feature = "fake_portal")]
use crate::portal::fake::PortalLibraryFake;
#[cfg(feature = "fake_process")]
use crate::process::fake::ProcessLibraryFake;
#[cfg(feature = "fake_random")]
use crate::random::fake::RandomLibraryFake;
#[cfg(feature = "fake_regex")]
use crate::regex::fake::RegexLibraryFake;
#[cfg(feature = "fake_report")]
use crate::report::fake::ReportLibraryFake;
#[cfg(feature = "fake_scan")]
use crate::scan::fake::ScanLibraryFake;
#[cfg(feature = "fake_ssh")]
use crate::ssh::fake::SshLibraryFake;
#[cfg(feature = "fake_sys")]
use crate::sys::fake::SysLibraryFake;
#[cfg(feature = "fake_template")]
use crate::template::fake::TemplateLibraryFake;
#[cfg(feature = "fake_time")]
use crate::time::fake::TimeLibraryFake;
#[cfg(feature = "fake_user")]
use crate::user::fake::UserLibraryFake;
#[cfg(feature = "fake_utf8")]
use crate::utf8::fake::Utf8LibraryFake;
#[cfg(feature = "fake_utf16le")]
use crate::utf16le::fake::Utf16leLibraryFake;
#[cfg(feature = "fake_windows")]
use crate::windows::fake::WindowsLibraryFake;

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

    #[cfg(feature = "stdlib")]
    pub fn with_default_libs(mut self) -> Self {
        {
            self.inner.register_lib(StdAesLibrary);
            self.inner.register_lib(StdBase64Library);
            self.inner.register_lib(StdCryptoLibrary);
            self.inner.register_lib(StdFileLibrary);
            self.inner.register_lib(StdGzipLibrary);
            self.inner.register_lib(StdHashLibrary);
            self.inner.register_lib(StdHttpLibrary);
            self.inner.register_lib(StdJsonLibrary);
            self.inner.register_lib(StdLinuxLibrary);
            self.inner.register_lib(StdNetLibrary);
            self.inner.register_lib(StdPivotLibrary::default());
            self.inner.register_lib(StdPortalLibrary::default());
            self.inner.register_lib(StdProcessLibrary);
            self.inner.register_lib(StdRandomLibrary);
            self.inner.register_lib(StdRegexLibrary);
            self.inner.register_lib(StdScanLibrary);
            self.inner.register_lib(StdSshLibrary);
            self.inner.register_lib(StdSysLibrary);
            self.inner.register_lib(StdTemplateLibrary);
            self.inner.register_lib(StdTimeLibrary);
            self.inner.register_lib(StdUserLibrary);
            self.inner.register_lib(StdUtf8Library);
            self.inner.register_lib(StdUtf16leLibrary);
            self.inner.register_lib(StdWindowsLibrary);
            self.inner.register_lib(StdDnsLibrary);
        }

        #[cfg(feature = "fake_aes")]
        self.inner.register_lib(AesLibraryFake);
        #[cfg(feature = "fake_base64")]
        self.inner.register_lib(Base64LibraryFake);
        #[cfg(feature = "fake_crypto")]
        self.inner.register_lib(CryptoLibraryFake);
        #[cfg(feature = "fake_file")]
        self.inner.register_lib(FileLibraryFake::default());
        #[cfg(feature = "fake_gzip")]
        self.inner.register_lib(GzipLibraryFake);
        #[cfg(feature = "fake_hash")]
        self.inner.register_lib(HashLibraryFake);
        #[cfg(feature = "fake_http")]
        self.inner.register_lib(HttpLibraryFake);
        #[cfg(feature = "fake_json")]
        self.inner.register_lib(JsonLibraryFake);
        #[cfg(feature = "fake_linux")]
        self.inner.register_lib(LinuxLibraryFake);
        #[cfg(feature = "fake_net")]
        self.inner.register_lib(NetLibraryFake);
        #[cfg(feature = "fake_pivot")]
        self.inner.register_lib(PivotLibraryFake);
        #[cfg(feature = "fake_portal")]
        self.inner.register_lib(PortalLibraryFake);
        #[cfg(feature = "fake_process")]
        self.inner.register_lib(ProcessLibraryFake);
        #[cfg(feature = "fake_random")]
        self.inner.register_lib(RandomLibraryFake);
        #[cfg(feature = "fake_regex")]
        self.inner.register_lib(RegexLibraryFake);
        #[cfg(feature = "fake_scan")]
        self.inner.register_lib(ScanLibraryFake);
        #[cfg(feature = "fake_ssh")]
        self.inner.register_lib(SshLibraryFake);
        #[cfg(feature = "fake_sys")]
        self.inner.register_lib(SysLibraryFake);
        #[cfg(feature = "fake_template")]
        self.inner.register_lib(TemplateLibraryFake);
        #[cfg(feature = "fake_time")]
        self.inner.register_lib(TimeLibraryFake);
        #[cfg(feature = "fake_user")]
        self.inner.register_lib(UserLibraryFake);
        #[cfg(feature = "fake_utf8")]
        self.inner.register_lib(Utf8LibraryFake);
        #[cfg(feature = "fake_utf16le")]
        self.inner.register_lib(Utf16leLibraryFake);
        #[cfg(feature = "fake_windows")]
        self.inner.register_lib(WindowsLibraryFake);
        #[cfg(feature = "fake_dns")]
        self.inner.register_lib(DnsLibraryFake::default());

        self
    }

    #[cfg(feature = "stdlib")]
    pub fn with_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        // Agent library needs a task_id. For general usage (outside of imix tasks),
        // we can use 0 or a placeholder.

        use pb::c2::TaskContext;
        let task_context = TaskContext {
            task_id: 0,
            jwt: String::new(),
        };
        let context = Context::Task(task_context.clone());

        let agent_lib = StdAgentLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(agent_lib);

        let report_lib = StdReportLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(report_lib);

        let pivot_lib = StdPivotLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(pivot_lib);

        let portal_lib = StdPortalLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(portal_lib);

        self.inner.register_lib(StdChainLibrary::new(agent.clone()));

        // Assets library
        let backend = Arc::new(crate::assets::std::AgentAssets::new(
            agent.clone(),
            context,
            Vec::new(),
        ));
        let mut assets_lib = StdAssetsLibrary::new();
        assets_lib.add_shadow(backend);
        self.inner.register_lib(assets_lib);

        self
    }

    #[allow(unused_mut)]
    pub fn with_fake_agent(mut self) -> Self {
        #[cfg(feature = "fake_agent")]
        self.inner.register_lib(AgentLibraryFake);
        #[cfg(feature = "fake_report")]
        self.inner.register_lib(ReportLibraryFake);
        #[cfg(feature = "fake_pivot")]
        self.inner.register_lib(PivotLibraryFake);
        #[cfg(feature = "fake_portal")]
        self.inner.register_lib(PortalLibraryFake);
        #[cfg(feature = "fake_assets")]
        self.inner.register_lib(FakeAssetsLibrary);
        #[cfg(feature = "fake_chain")]
        self.inner.register_lib(FakeChainLibrary::new());
        self
    }

    #[cfg(feature = "stdlib")]
    pub fn with_context(
        mut self,
        agent: Arc<dyn Agent>,
        context: Context,
        remote_assets: Vec<String>,
        backend: Arc<dyn assets::std::AssetBackend>,
    ) -> Self {
        let agent_lib = StdAgentLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(agent_lib);

        let report_lib = StdReportLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(report_lib);

        let pivot_lib = StdPivotLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(pivot_lib);

        let portal_lib = StdPortalLibrary::new(agent.clone(), context.clone());
        self.inner.register_lib(portal_lib);

        self.inner.register_lib(StdChainLibrary::new(agent.clone()));

        let mut assets_lib = StdAssetsLibrary::new();
        // As with previously, remote assets can shadow the Embedded Assets
        let agent_backend = Arc::new(crate::assets::std::AgentAssets::new(
            agent.clone(),
            context,
            remote_assets.clone(),
        ));
        assets_lib.add_shadow(agent_backend.clone());
        assets_lib.add_shadow(backend.clone());
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

#[cfg(test)]
mod process_report_test;

#[cfg(all(test, feature = "stdlib"))]
mod rename_test;
