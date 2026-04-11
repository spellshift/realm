use super::DnsLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[eldritch_library_impl(DnsLibrary)]
#[derive(Debug, Default)]
pub struct DnsLibraryFake;

impl DnsLibrary for DnsLibraryFake {
    fn list(
        &self,
        _domain: String,
        record_type: Option<String>,
        _nameserver: Option<String>,
    ) -> Result<Vec<String>, String> {
        let rtype = record_type.unwrap_or_else(|| alloc::string::String::from("A")).to_uppercase();
        match rtype.as_str() {
            "A" => Ok(alloc::vec!["127.0.0.1".into(), "10.0.0.1".into()]),
            "CNAME" => Ok(alloc::vec!["alias.example.com".into()]),
            _ => Err(alloc::format!("Unsupported record type: {}", rtype)),
        }
    }
}
