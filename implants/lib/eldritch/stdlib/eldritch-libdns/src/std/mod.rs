use super::DnsLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use hickory_resolver::Resolver;
use hickory_resolver::config::*;

#[eldritch_library_impl(DnsLibrary)]
#[derive(Debug, Default)]
pub struct StdDnsLibrary;

impl DnsLibrary for StdDnsLibrary {
    fn list_a_records(&self, domain: String, nameserver: Option<String>) -> Result<Vec<String>, String> {
        let mut config = ResolverConfig::default();
        if let Some(ns) = nameserver {
            use std::str::FromStr;
            let ns_ip = std::net::IpAddr::from_str(&ns)
                .map_err(|e| format!("Invalid nameserver IP {}: {}", ns, e))?;
            let ns_addr = std::net::SocketAddr::new(ns_ip, 53);
            let ns_config = NameServerConfig::new(ns_addr, Protocol::Udp);
            config = ResolverConfig::from_parts(None, alloc::vec::Vec::new(), NameServerConfigGroup::from(alloc::vec![ns_config]));
        }

        let resolver = Resolver::new(config, ResolverOpts::default())
            .map_err(|e| format!("Failed to create resolver: {}", e))?;

        // This is a blocking resolution. For an A record, we only want IPv4
        let response = resolver
            .ipv4_lookup(&domain)
            .map_err(|e| format!("Failed to resolve {}: {}", domain, e))?;

        let ips: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();

        Ok(ips)
    }
}
