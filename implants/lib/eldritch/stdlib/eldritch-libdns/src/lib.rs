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
    /// Resolves DNS records for a domain.
    ///
    /// **Parameters**
    /// - `domain` (`str`): The domain name to resolve.
    /// - `record_type` (`Option<str>`): An optional record type to query ("A" or "CNAME"). Defaults to "A".
    /// - `nameserver` (`Option<str>`): An optional nameserver IP to query (e.g. "8.8.8.8").
    ///
    /// **Returns**
    /// - `List<str>`: A list of result strings depending on the record type.
    ///
    /// **Errors**
    /// - Returns an error string if resolution fails or the record type is unsupported.
    fn list(
        &self,
        domain: String,
        record_type: Option<String>,
        nameserver: Option<String>,
    ) -> Result<Vec<String>, String>;
}
