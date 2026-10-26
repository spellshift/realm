extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_deprecated, eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("pivot")]
/// The `pivot` library is deprecated; its functions have been moved to `scan`, `portal`, `net`, and `ssh`.
pub trait PivotLibrary {
    #[eldritch_method]
    #[eldritch_deprecated(
        "pivot.create_portal has been deprecated and will be removed in a future version of eldritch. Please use portal.create instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Opens a portal bi-directional stream.
    fn create_portal(&self) -> Result<(), String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    #[eldritch_deprecated(
        "pivot.ssh_exec has been deprecated and will be removed in a future version of eldritch. Please use ssh.exec instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
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
    #[eldritch_deprecated(
        "pivot.ssh_copy has been deprecated and will be removed in a future version of eldritch. Please use ssh.copy instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
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
    #[eldritch_deprecated(
        "pivot.port_scan has been deprecated and will be removed in a future version of eldritch. Please use scan.port instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
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
    #[eldritch_deprecated(
        "pivot.arp_scan has been deprecated and will be removed in a future version of eldritch. Please use scan.arp instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Performs an ARP scan to discover live hosts on the local network.
    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[allow(clippy::too_many_arguments)]
    #[eldritch_method]
    #[eldritch_deprecated(
        "pivot.ssh_deploy has been deprecated and will be removed in a future version of eldritch. Please use ssh.deploy instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Deploys a payload and/or command across a set of hosts via SSH.
    fn ssh_deploy(
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

    #[eldritch_method]
    #[eldritch_deprecated(
        "pivot.ncat has been deprecated and will be removed in a future version of eldritch. Please use net.tcp & net.udp instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Sends arbitrary data to a host via TCP or UDP and waits for a response.
    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String>;
}
