#![allow(clippy::mutable_key_type)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("dns")]
/// The `dns` library enables the agent to make DNS queries.
pub trait DnsLibrary {
    #[eldritch_method]
    /// Resolves the A records for a domain.
    ///
    /// **Parameters**
    /// - `domain` (`str`): The domain name to resolve.
    /// - `nameserver` (`Option<str>`): An optional nameserver IP to query (e.g. "8.8.8.8").
    ///
    /// **Returns**
    /// - `List<str>`: A list of IPv4 addresses.
    ///
    /// **Errors**
    /// - Returns an error string if resolution fails.
    fn list_a_records(&self, domain: String, nameserver: Option<String>) -> Result<Vec<String>, String>;
}
