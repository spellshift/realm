use super::DnsLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[eldritch_library_impl(DnsLibrary)]
#[derive(Debug, Default)]
pub struct DnsLibraryFake;

impl DnsLibrary for DnsLibraryFake {
    fn resolve_a(&self, _domain: String, _nameserver: Option<String>) -> Result<Vec<String>, String> {
        // Return dummy IPs for testing
        Ok(alloc::vec!["127.0.0.1".into(), "10.0.0.1".into()])
    }
}
