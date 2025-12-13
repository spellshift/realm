extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[cfg(feature = "stdlib")]
pub mod arp_scan_impl;
#[cfg(feature = "stdlib")]
pub mod ncat_impl;
#[cfg(feature = "stdlib")]
pub mod port_scan_impl;
#[cfg(feature = "stdlib")]
pub mod reverse_shell_pty_impl;
#[cfg(feature = "stdlib")]
pub mod ssh_copy_impl;
#[cfg(feature = "stdlib")]
pub mod ssh_exec_impl;

#[cfg(test)]
mod tests;

pub trait ReplHandler: Send + Sync {
    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String>;
}

#[eldritch_library("pivot")]
pub trait PivotLibrary {
    #[eldritch_method]
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    fn reverse_shell_repl(&self) -> Result<(), String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    fn ssh_exec(
        &self,
        target: String,
        port: i64,
        command: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    fn ssh_copy(
        &self,
        target: String,
        port: i64,
        src: String,
        dst: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<String, String>;

    #[eldritch_method]
    fn port_scan(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String>;
}
