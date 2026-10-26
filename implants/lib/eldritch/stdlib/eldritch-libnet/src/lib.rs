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

#[eldritch_library("net")]
/// The `net` library provides networking utilities including interface listing, active socket inspection, and raw TCP/UDP communication.
pub trait NetLibrary {
    #[eldritch_method]
    /// Returns a list of network interfaces and their IP addresses.
    ///
    /// **Returns**
    /// - `List<Dict<str, str>>`: List of dicts with `name` and `ip`.
    fn interfaces(&self) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    /// Returns a list of active network connections (TCP/UDP/Unix).
    ///
    /// **Returns**
    /// - `List<Dict>`: A list of connection details including socket type, local/remote address/port, and associated PID.
    fn conns(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Sends arbitrary data to a host via TCP and waits for a response.
    ///
    /// **Parameters**
    /// - `address` (`str`): Target address.
    /// - `port` (`int`): Target port.
    /// - `data` (`str`): Data to send.
    ///
    /// **Returns**
    /// - `str`: The response data.
    fn tcp(&self, address: String, port: i64, data: String) -> Result<String, String>;

    #[eldritch_method]
    /// Sends arbitrary data to a host via UDP and waits for a response.
    ///
    /// **Parameters**
    /// - `address` (`str`): Target address.
    /// - `port` (`int`): Target port.
    /// - `data` (`str`): Data to send.
    ///
    /// **Returns**
    /// - `str`: The response data.
    fn udp(&self, address: String, port: i64, data: String) -> Result<String, String>;
}
