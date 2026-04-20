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

#[cfg(test)]
mod tests;

#[eldritch_library("pivot")]
/// The `pivot` library provides tools for lateral movement, scanning, and tunneling.
///
/// It supports:
/// - Reverse shells (PTY and REPL).
/// - SSH execution and file copy.
/// - Network scanning (ARP, Port).
/// - Traffic tunneling (Port forwarding, Bind proxy).
/// - Simple network interaction (Ncat).
/// - SMB execution (Stubbed/Proposed).
pub trait PivotLibrary {
    #[eldritch_method]
    /// Spawns a reverse shell with a PTY (Pseudo-Terminal) attached.
    ///
    /// This provides a full interactive shell experience over the agent's C2 channel.
    ///
    /// **Parameters**
    /// - `cmd` (`Option<str>`): The shell command to run (e.g., `/bin/bash`, `cmd.exe`). If `None`, defaults to system shell.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the shell cannot be spawned.
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    /// Spawns a basic REPL-style reverse shell with an Eldritch interpreter.
    ///
    /// Useful if PTY is not available.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if failure occurs.
    fn reverse_shell_repl(&self) -> Result<(), String>;

    #[eldritch_method]
    /// Opens a portal bi-directional stream.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if failure occurs.
    fn create_portal(&self) -> Result<(), String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    /// Executes a command on a remote host via SSH.
    ///
    /// **Parameters**
    /// - `target` (`str`): The remote host IP or hostname.
    /// - `port` (`int`): The SSH port (usually 22).
    /// - `command` (`str`): The command to execute.
    /// - `username` (`str`): SSH username.
    /// - `password` (`Option<str>`): SSH password (optional).
    /// - `key` (`Option<str>`): SSH private key (optional).
    /// - `key_password` (`Option<str>`): Password for the private key (optional).
    /// - `timeout` (`Option<int>`): Connection timeout in seconds (optional).
    ///
    /// **Returns**
    /// - `Dict`: A dictionary containing command output:
    ///   - `stdout` (`str`)
    ///   - `stderr` (`str`)
    ///   - `status` (`int`): Exit code.
    ///
    /// **Errors**
    /// - Returns an error string if connection fails.
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
    ///
    /// **Parameters**
    /// - `target` (`str`): The remote host IP or hostname.
    /// - `port` (`int`): The SSH port.
    /// - `src` (`str`): Local source file path.
    /// - `dst` (`str`): Remote destination file path.
    /// - `username` (`str`): SSH username.
    /// - `password` (`Option<str>`): SSH password.
    /// - `key` (`Option<str>`): SSH private key.
    /// - `key_password` (`Option<str>`): Key password.
    /// - `timeout` (`Option<int>`): Connection timeout.
    ///
    /// **Returns**
    /// - `str`: "Success" message or error detail.
    ///
    /// **Errors**
    /// - Returns an error string if copy fails.
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
    ///
    /// **Parameters**
    /// - `target_cidrs` (`List<str>`): List of CIDRs to scan (e.g., `["192.168.1.0/24"]`).
    /// - `ports` (`List<int>`): List of ports to scan.
    /// - `protocol` (`str`): "tcp" or "udp".
    /// - `timeout` (`int`): Timeout per port in seconds.
    /// - `fd_limit` (`Option<int>`): Maximum concurrent file descriptors/sockets (defaults to 64).
    ///
    /// **Returns**
    /// - `List<Dict>`: List of open ports/results.
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
    ///
    /// **Parameters**
    /// - `target_cidrs` (`List<str>`): List of CIDRs to scan.
    ///
    /// **Returns**
    /// - `List<Dict>`: List of discovered hosts with IP, MAC, and Interface.
    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    /// Deploys a payload and/or command across a set of hosts via SSH.
    ///
    /// For each target (IP or CIDR range), this method attempts the provided
    /// credentials in order until one succeeds. Once authenticated, an optional
    /// payload is copied via SFTP to the destination path, and then `cmd` is
    /// executed. If the effective user is not `root` and `privesc_cmd` is
    /// provided, the privilege escalation command is executed before `cmd`.
    ///
    /// **Parameters**
    /// - `ips` (`List<str>`): Non-empty list of IP addresses and/or CIDR ranges
    ///   (e.g. `["10.0.0.1", "10.0.0.0/24"]`). All entries must be valid.
    /// - `credentials` (`List<Dict>`): Non-empty list of credential dictionaries
    ///   of the form `{"principal": "<user>", "password": "<password>"}`,
    ///   attempted in order on each host.
    /// - `cmd` (`str`): Command to run on the remote system (ideally as root).
    /// - `privesc_cmd` (`Option<str>`): Optional privilege escalation command
    ///   to run when the effective user is not root.
    /// - `payload` (`Option<str>`): Optional local path to a binary payload to
    ///   copy to the remote system.
    /// - `payload_dst` (`Option<str>`): Optional destination path on the remote
    ///   system for the payload. Defaults to `/tmp/<basename(payload)>`.
    ///
    /// **Returns**
    /// - `List<Dict>`: One result dictionary per target IP with the keys:
    ///   - `ip` (`str`)
    ///   - `status` (`str`): `"success"` or `"failed"`
    ///   - `principal` (`str`): Credential principal used on success (empty on failure).
    ///   - `stdout` (`str`)
    ///   - `stderr` (`str`)
    ///   - `error` (`str`): Error detail on failure (empty on success).
    ///
    /// **Errors**
    /// - Returns an error string if `ips` or `credentials` is empty, or if any
    ///   entry in `ips` or `credentials` is malformed.
    fn ssh_deploy(
        &self,
        ips: Vec<String>,
        credentials: Vec<BTreeMap<String, Value>>,
        cmd: String,
        privesc_cmd: Option<String>,
        payload: Option<String>,
        payload_dst: Option<String>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Sends arbitrary data to a host via TCP or UDP and waits for a response.
    ///
    /// **Parameters**
    /// - `address` (`str`): Target address.
    /// - `port` (`int`): Target port.
    /// - `data` (`str`): Data to send.
    /// - `protocol` (`str`): "tcp" or "udp".
    ///
    /// **Returns**
    /// - `str`: The response data.
    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String>;
}
