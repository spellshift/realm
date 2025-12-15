extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::{Interpreter, Value};
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

#[eldritch_library("pivot")]
/// The `pivot` library provides tools for lateral movement, scanning, and tunneling.
pub trait PivotLibrary {
    #[eldritch_method]
    /// Spawns a reverse shell with a PTY (Pseudo-Terminal) attached.
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    /// Spawns a basic REPL-style reverse shell.
    fn reverse_shell_repl(&self, interp: &mut Interpreter) -> Result<(), String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    /// Executes a command on a remote host via SSH.
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
    /// Copies a file to a remote host via SSH (SCP/SFTP).
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
    /// Scans TCP/UDP ports on target hosts.
    fn port_scan(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Performs an ARP scan to discover live hosts on the local network.
    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Sends arbitrary data to a host via TCP or UDP and waits for a response.
    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String>;
}
