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
    fn list(
        &self,
        domain: String,
        kind: Option<String>,
        nameserver: Option<String>,
    ) -> Result<Vec<String>, String> {
        let mut config = ResolverConfig::default();
        if let Some(ns) = nameserver {
            use std::str::FromStr;
            let ns_ip = std::net::IpAddr::from_str(&ns)
                .map_err(|e| format!("Invalid nameserver IP {}: {}", ns, e))?;
            let ns_addr = std::net::SocketAddr::new(ns_ip, 53);
            let ns_config = NameServerConfig::new(ns_addr, Protocol::Udp);
            config = ResolverConfig::from_parts(
                None,
                alloc::vec::Vec::new(),
                NameServerConfigGroup::from(alloc::vec![ns_config]),
            );
        }

        let resolver = Resolver::new(config, ResolverOpts::default())
            .map_err(|e| format!("Failed to create resolver: {}", e))?;

        let rtype = kind.unwrap_or_else(|| alloc::string::String::from("A"));
        match rtype.to_uppercase().as_str() {
            "A" => resolve_a_records(&resolver, &domain),
            "AAAA" => resolve_aaaa_records(&resolver, &domain),
            "CNAME" => resolve_cname_records(&resolver, &domain),
            "TXT" => resolve_txt_records(&resolver, &domain),
            "MX" => resolve_mx_records(&resolver, &domain),
            "SOA" => resolve_soa_records(&resolver, &domain),
            "NS" => resolve_ns_records(&resolver, &domain),
            "PTR" => resolve_ptr_records(&resolver, &domain),
            "AXFR" => resolve_axfr_records(&resolver, &domain),
            "SRV" => resolve_srv_records(&resolver, &domain),
            _ => Err(format!("Unsupported record type: {}", rtype)),
        }
    }
}

fn resolve_a_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .ipv4_lookup(domain)
        .map_err(|e| format!("Failed to resolve A records for {}: {}", domain, e))?;
    Ok(response.iter().map(|ip| ip.to_string()).collect())
}

fn resolve_aaaa_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .ipv6_lookup(domain)
        .map_err(|e| format!("Failed to resolve AAAA records for {}: {}", domain, e))?;
    Ok(response.iter().map(|ip| ip.to_string()).collect())
}

fn resolve_cname_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME)
        .map_err(|e| format!("Failed to resolve CNAME for {}: {}", domain, e))?;
    Ok(response.iter().map(|ip| ip.to_string()).collect())
}

fn resolve_txt_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .txt_lookup(domain)
        .map_err(|e| format!("Failed to resolve TXT for {}: {}", domain, e))?;
    Ok(response.iter().map(|t| t.to_string()).collect())
}

fn resolve_mx_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .mx_lookup(domain)
        .map_err(|e| format!("Failed to resolve MX for {}: {}", domain, e))?;
    Ok(response.iter().map(|m| m.to_string()).collect())
}

fn resolve_soa_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::SOA)
        .map_err(|e| format!("Failed to resolve SOA for {}: {}", domain, e))?;
    Ok(response.iter().map(|r| r.to_string()).collect())
}

fn resolve_ns_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::NS)
        .map_err(|e| format!("Failed to resolve NS for {}: {}", domain, e))?;
    Ok(response.iter().map(|r| r.to_string()).collect())
}

fn resolve_ptr_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::PTR)
        .map_err(|e| format!("Failed to resolve PTR for {}: {}", domain, e))?;
    Ok(response.iter().map(|r| r.to_string()).collect())
}

fn resolve_axfr_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::AXFR)
        .map_err(|e| format!("Failed to resolve AXFR for {}: {}", domain, e))?;
    Ok(response.iter().map(|r| r.to_string()).collect())
}

fn resolve_srv_records(resolver: &Resolver, domain: &str) -> Result<Vec<String>, String> {
    let response = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::SRV)
        .map_err(|e| format!("Failed to resolve SRV for {}: {}", domain, e))?;
    Ok(response.iter().map(|r| r.to_string()).collect())
}
