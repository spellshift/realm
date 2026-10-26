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

#[eldritch_library("ssh")]
/// The `ssh` library provides SSH command execution, file copy, and multi-host deployment.
pub trait SshLibrary {
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
    fn copy(
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

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    /// Deploys a payload and/or command across a set of hosts via SSH.
    ///
    /// **Parameters**
    /// - `ips` (`List<str>`): Non-empty list of IP addresses and/or CIDR ranges.
    /// - `credentials` (`List<Dict>`): Non-empty list of credential dictionaries.
    /// - `cmd` (`str`): Command to run on the remote system (ideally as root).
    /// - `privesc_cmd` (`Option<str>`): Optional privilege escalation command.
    /// - `payload` (`Option<bytes>`): Optional raw payload bytes to copy to the remote system.
    /// - `payload_dst` (`Option<str>`): Optional destination path on the remote system for the payload.
    /// - `timeout` (`Option<int>`): Optional per-connection timeout in seconds.
    /// - `retries` (`Option<int>`): Optional number of additional retry passes.
    ///
    /// **Returns**
    /// - `List<Dict>`: One result dictionary per target IP.
    fn deploy(
        &self,
        ips: Vec<String>,
        credentials: Vec<BTreeMap<String, Value>>,
        cmd: String,
        privesc_cmd: Option<String>,
        payload: Option<Vec<u8>>,
        payload_dst: Option<String>,
        timeout: Option<i64>,
        retries: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String>;

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
    fn exec(
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
}
