extern crate alloc;

use eldritch_macros::{eldritch_library, eldritch_method};

pub mod tcp_impl;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("chain")]
/// The `chain` library provides TCP chaining capabilities.
pub trait ChainLibrary {
    #[eldritch_method]
    /// Starts the agent proxy serving C2 over a TCP connection.
    ///
    /// **Parameters**
    /// - `addr` (`str`): The TCP address to connect to (e.g., "192.168.1.5:8443").
    ///
    /// **Returns**
    /// - `int` Error code or 0 on success.
    fn tcp(&self, addr: String) -> Result<i64, String>;
}
