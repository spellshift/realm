extern crate alloc;
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod arp_scan_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod bind_proxy_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod ncat_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod port_forward_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod port_scan_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod reverse_shell_pty_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod smb_exec_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod ssh_copy_impl;
#[cfg(any(feature = "stdlib", feature = "fake_bindings"))]
pub mod ssh_exec_impl;

#[cfg(test)]
mod tests;

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
    fn smb_exec(
        &self,
        target: String,
        port: i64,
        username: String,
        password: String,
        hash: String,
        command: String,
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
    fn port_forward(
        &self,
        listen_address: String,
        listen_port: i64,
        forward_address: String,
        forward_port: i64,
        protocol: String,
    ) -> Result<(), String>;

    #[eldritch_method]
    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String>;

    #[eldritch_method]
    fn bind_proxy(
        &self,
        listen_address: String,
        listen_port: i64,
        username: String,
        password: String,
    ) -> Result<(), String>;
}
