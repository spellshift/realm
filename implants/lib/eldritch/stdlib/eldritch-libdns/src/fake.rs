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
        kind: Option<String>,
        _nameserver: Option<String>,
    ) -> Result<Vec<String>, String> {
        let rtype = kind
            .unwrap_or_else(|| alloc::string::String::from("A"))
            .to_uppercase();
        match rtype.as_str() {
            "A" => Ok(alloc::vec!["127.0.0.1".into(), "10.0.0.1".into()]),
            "AAAA" => Ok(alloc::vec!["::1".into()]),
            "CNAME" => Ok(alloc::vec!["alias.example.com".into()]),
            "TXT" => Ok(alloc::vec!["v=spf1 -all".into()]),
            "MX" => Ok(alloc::vec!["10 mail.example.com".into()]),
            "SOA" => Ok(alloc::vec![
                "ns1.example.com admin.example.com 2026 7200 3600 1209600 3600".into()
            ]),
            "NS" => Ok(alloc::vec!["ns1.example.com".into()]),
            "PTR" => Ok(alloc::vec!["google.com".into()]),
            "AXFR" => Ok(alloc::vec![
                "example.com 86400 IN SOA ns1.example.com".into()
            ]),
            "SRV" => Ok(alloc::vec!["10 50 5060 sip.example.com".into()]),
            _ => Err(alloc::format!("Unsupported record type: {}", rtype)),
        }
    }
}
