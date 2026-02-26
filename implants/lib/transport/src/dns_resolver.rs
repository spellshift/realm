//! DNS resolution module for gRPC transport
//!
//! This module provides DNS-over-HTTPS (DoH) support for gRPC connections
//! when the `doh` feature is enabled.

#[cfg(feature = "doh")]
pub mod doh {
    use hickory_resolver::config::{
        NameServerConfig, Protocol, ResolverConfig, ResolverOpts,
    };
    use hickory_resolver::Name as HickoryName;
    use hickory_resolver::TokioAsyncResolver;
    use hyper_legacy::client::connect::dns::Name;
    use hyper_legacy::client::HttpConnector;
    use hyper_legacy::service::Service;
    #[cfg(feature = "grpc")]
    use hyper_util::client::legacy::connect::dns::Name as Hyper1Name;
    #[cfg(feature = "grpc")]
    use hyper_util::client::legacy::connect::HttpConnector as Hyper1HttpConnector;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    #[cfg(feature = "grpc")]
    use tower::Service as TowerService;

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy)]
    pub enum DohProvider {
        Cloudflare,
        Google,
        Quad9,
        System, // Use system DNS configuration
    }

    pub(crate) fn parse_resolv_conf(content: &str) -> (Vec<SocketAddr>, Vec<String>) {
        let mut addrs = Vec::new();
        let mut search = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("nameserver") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ip_str = parts[1];
                    // Try to parse as IP address, default port 53
                    if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                        addrs.push(SocketAddr::new(ip, 53));
                    }
                }
            } else if line.starts_with("search") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    search = parts[1..].iter().map(|s| s.to_string()).collect();
                }
            } else if line.starts_with("domain") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    search = vec![parts[1].to_string()];
                }
            }
        }
        (addrs, search)
    }

    impl DohProvider {
        fn resolver_config(&self) -> Result<ResolverConfig, anyhow::Error> {
            match self {
                DohProvider::Cloudflare => Ok(ResolverConfig::cloudflare_https()),
                DohProvider::Google => Ok(ResolverConfig::google_https()),
                DohProvider::Quad9 => Ok(ResolverConfig::quad9_https()),
                DohProvider::System => {
                    // Read system DNS configuration
                    match hickory_resolver::system_conf::read_system_conf() {
                        Ok((config, _opts)) => Ok(config),
                        Err(e) => {
                            log::warn!(
                                "Failed to read system DNS config: {}. Attempting manual parsing.",
                                e
                            );

                            let mut nameservers = Vec::new();
                            let mut search = Vec::new();

                            // Try to read /etc/resolv.conf manually
                            if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
                                let (ns, s) = parse_resolv_conf(&content);
                                nameservers = ns;
                                search = s;
                            }

                            if nameservers.is_empty() {
                                log::warn!("Manual parsing failed or found no nameservers. Falling back to 1.1.1.1 and 8.8.8.8");
                                nameservers.push("1.1.1.1:53".parse().unwrap());
                                nameservers.push("8.8.8.8:53".parse().unwrap());
                            } else {
                                log::info!(
                                    "Manual parsing found {} nameservers and {} search domains.",
                                    nameservers.len(),
                                    search.len()
                                );
                            }

                            let mut ns_config_group = Vec::new();
                            for ns in nameservers {
                                ns_config_group
                                    .push(NameServerConfig::new(ns, Protocol::Udp));
                                ns_config_group
                                    .push(NameServerConfig::new(ns, Protocol::Tcp));
                            }

                            let search_list: Vec<HickoryName> = search
                                .iter()
                                .filter_map(|s| {
                                    use std::str::FromStr;
                                    HickoryName::from_str(s).ok()
                                })
                                .collect();

                            let domain = search_list.first().cloned();

                            // Use from_parts to fully construct the config with search domains
                            Ok(ResolverConfig::from_parts(
                                domain,
                                search_list,
                                ns_config_group,
                            ))
                        }
                    }
                }
            }
        }
    }

    /// Wrapper around hickory-resolver's TokioAsyncResolver that implements
    /// hyper's Service<Name> trait for DNS resolution
    #[derive(Clone)]
    pub struct HickoryResolverService {
        resolver: TokioAsyncResolver,
    }

    impl HickoryResolverService {
        /// Create a new resolver service with the specified provider (DOH or system DNS)
        pub fn new(provider: DohProvider) -> Result<Self, anyhow::Error> {
            let config = provider.resolver_config()?;
            let opts = ResolverOpts::default();

            let resolver = TokioAsyncResolver::tokio(config, opts);

            Ok(Self { resolver })
        }
    }

    impl Service<Name> for HickoryResolverService {
        type Response = HickoryAddressIter;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, name: Name) -> Self::Future {
            let resolver = self.resolver.clone();

            let name_str = name.as_str().to_string();

            Box::pin(async move {
                let lookup = resolver
                    .lookup_ip(&name_str)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                let addrs: Vec<SocketAddr> =
                    lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect();

                Ok(HickoryAddressIter {
                    addrs: addrs.into_iter(),
                })
            })
        }
    }

    /// Wrapper around hickory-resolver's TokioAsyncResolver that implements
    /// tower's Service<Name> trait for DNS resolution (Hyper 1.x / hyper-util)
    #[cfg(feature = "grpc")]
    #[derive(Clone)]
    pub struct HickoryResolverServiceHyper1 {
        resolver: TokioAsyncResolver,
    }

    #[cfg(feature = "grpc")]
    impl HickoryResolverServiceHyper1 {
        /// Create a new resolver service with the specified provider (DOH or system DNS)
        pub fn new(provider: DohProvider) -> Result<Self, anyhow::Error> {
            let config = provider.resolver_config()?;
            let opts = ResolverOpts::default();

            let resolver = TokioAsyncResolver::tokio(config, opts);

            Ok(Self { resolver })
        }
    }

    #[cfg(feature = "grpc")]
    impl TowerService<Hyper1Name> for HickoryResolverServiceHyper1 {
        type Response = HickoryAddressIter;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, name: Hyper1Name) -> Self::Future {
            let resolver = self.resolver.clone();

            let name_str = name.as_str().to_string();

            Box::pin(async move {
                let lookup = resolver
                    .lookup_ip(&name_str)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                let addrs: Vec<SocketAddr> =
                    lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect();

                Ok(HickoryAddressIter {
                    addrs: addrs.into_iter(),
                })
            })
        }
    }

    /// Iterator over resolved socket addresses
    pub struct HickoryAddressIter {
        addrs: std::vec::IntoIter<SocketAddr>,
    }

    impl Iterator for HickoryAddressIter {
        type Item = SocketAddr;

        fn next(&mut self) -> Option<Self::Item> {
            self.addrs.next()
        }
    }

    /// Create an HTTP connector with DoH support using the specified provider
    pub fn create_doh_connector(
        provider: DohProvider,
    ) -> Result<HttpConnector<HickoryResolverService>, anyhow::Error> {
        let resolver = HickoryResolverService::new(provider)?;
        Ok(HttpConnector::new_with_resolver(resolver))
    }

    /// Create a hyper-util (Hyper 1.x) HTTP connector with DoH support using the specified provider
    #[cfg(feature = "grpc")]
    pub fn create_doh_connector_hyper1(
        provider: DohProvider,
    ) -> Result<Hyper1HttpConnector<HickoryResolverServiceHyper1>, anyhow::Error> {
        let resolver = HickoryResolverServiceHyper1::new(provider)?;
        Ok(Hyper1HttpConnector::new_with_resolver(resolver))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "doh")]
    use super::doh::*;

    #[cfg(feature = "doh")]
    #[tokio::test]
    async fn test_doh_resolver_creation() {
        let result = HickoryResolverService::new(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH resolver");
    }

    #[cfg(feature = "doh")]
    #[tokio::test]
    async fn test_doh_connector_creation() {
        let result = create_doh_connector(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH connector");
    }

    #[cfg(all(feature = "doh", feature = "grpc"))]
    #[tokio::test]
    async fn test_doh_resolver_hyper1_creation() {
        let result = HickoryResolverServiceHyper1::new(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH resolver (Hyper 1.x)");
    }

    #[cfg(all(feature = "doh", feature = "grpc"))]
    #[tokio::test]
    async fn test_doh_connector_hyper1_creation() {
        let result = create_doh_connector_hyper1(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH connector (Hyper 1.x)");
    }

    #[cfg(feature = "doh")]
    #[tokio::test]
    async fn test_dns_resolution() {
        use hyper_legacy::client::connect::dns::Name;
        use hyper_legacy::service::Service;
        use std::str::FromStr;

        let mut resolver = HickoryResolverService::new(DohProvider::Cloudflare)
            .expect("Failed to create resolver");

        let name = Name::from_str("google.com").expect("Failed to create Name");
        let result = resolver.call(name).await;

        match &result {
            Ok(_) => {
                let addrs: Vec<_> = result.unwrap().collect();
                println!("addrs: {:?}", addrs);
                assert!(!addrs.is_empty(), "No addresses resolved");
            }
            Err(e) => {
                panic!("DNS resolution failed with error: {}", e);
            }
        }
    }

    #[cfg(feature = "doh")]
    #[test]
    fn test_parse_resolv_conf() {
        use std::net::SocketAddr;
        use std::str::FromStr;
        let content = r#"
# Some comments
nameserver 8.8.8.8
nameserver 1.1.1.1
unknown_directive foo bar
nameserver invalid_ip
search corp.local internal.net
"#;
        let (addrs, search) = parse_resolv_conf(content);
        assert_eq!(addrs.len(), 2);
        assert_eq!(addrs[0], SocketAddr::from_str("8.8.8.8:53").unwrap());
        assert_eq!(addrs[1], SocketAddr::from_str("1.1.1.1:53").unwrap());

        assert_eq!(search.len(), 2);
        assert_eq!(search[0], "corp.local");
        assert_eq!(search[1], "internal.net");
    }

    #[cfg(feature = "doh")]
    #[test]
    fn test_parse_resolv_conf_domain() {
        let content = r#"
nameserver 8.8.8.8
domain example.com
"#;
        let (_, search) = parse_resolv_conf(content);
        assert_eq!(search.len(), 1);
        assert_eq!(search[0], "example.com");
    }

    #[cfg(feature = "doh")]
    #[test]
    fn test_parse_resolv_conf_precedence() {
        // Last one wins
        let content = r#"
search old.search
domain example.com
search new.search another.search
"#;
        let (_, search) = parse_resolv_conf(content);
        assert_eq!(search.len(), 2);
        assert_eq!(search[0], "new.search");
        assert_eq!(search[1], "another.search");
    }
}
