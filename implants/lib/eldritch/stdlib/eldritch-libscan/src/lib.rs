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

#[eldritch_library("scan")]
/// The `scan` library provides network discovery and port scanning tools.
pub trait ScanLibrary {
    #[eldritch_method]
    /// Performs an ARP scan to discover live hosts on the local network.
    ///
    /// **Parameters**
    /// - `target_cidrs` (`List<str>`): List of CIDRs to scan.
    ///
    /// **Returns**
    /// - `List<Dict>`: List of discovered hosts with IP, MAC, and Interface.
    fn arp(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String>;

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
    fn ports(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String>;
}
