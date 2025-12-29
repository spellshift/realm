//! DNS resolution module for gRPC transport
//!
//! This module provides DNS-over-HTTPS (DoH) support for gRPC connections
//! at runtime via URI query parameters.

pub mod doh {
    use hickory_resolver::config::{ResolverConfig, ResolverOpts};
    use hickory_resolver::TokioAsyncResolver;
    use hyper::client::connect::dns::Name;
    use hyper::client::HttpConnector;
    use hyper::service::Service;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    pub enum DohProvider {
        Cloudflare,
        Google,
        Quad9,
        Custom(String),
    }

    impl DohProvider {
        fn resolver_config(&self) -> Result<ResolverConfig, anyhow::Error> {
            match self {
                DohProvider::Cloudflare => Ok(ResolverConfig::cloudflare_https()),
                DohProvider::Google => Ok(ResolverConfig::google_https()),
                DohProvider::Quad9 => Ok(ResolverConfig::quad9_https()),
                DohProvider::Custom(url) => {
                    // For custom DoH endpoints, we would need to parse the URL and construct a config
                    // For now, fall back to Cloudflare as a sensible default
                    // TODO: Implement proper custom DoH URL parsing
                    log::warn!(
                        "Custom DoH endpoint support not yet implemented: {}. Using Cloudflare.",
                        url
                    );
                    Ok(ResolverConfig::cloudflare_https())
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
        /// Create a new resolver service with the specified DoH provider
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
        let mut http = HttpConnector::new_with_resolver(resolver);
        http.enforce_http(false);
        http.set_nodelay(true);
        Ok(http)
    }

    /// Create an HTTP connector with system DNS (no DoH)
    pub fn create_system_dns_connector(
    ) -> Result<HttpConnector<HickoryResolverService>, anyhow::Error> {
        use hickory_resolver::system_conf::read_system_conf;
        let (config, opts) = read_system_conf()?;
        let resolver = TokioAsyncResolver::tokio(config, opts);
        let resolver_service = HickoryResolverService { resolver };
        let mut http = HttpConnector::new_with_resolver(resolver_service);
        http.enforce_http(false);
        http.set_nodelay(true);
        Ok(http)
    }
}

#[cfg(test)]
mod tests {
    use super::doh::*;

    #[tokio::test]
    async fn test_doh_resolver_creation() {
        let result = HickoryResolverService::new(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH resolver");
    }

    #[tokio::test]
    async fn test_doh_connector_creation() {
        let result = create_doh_connector(DohProvider::Cloudflare);
        assert!(result.is_ok(), "Failed to create DoH connector");
    }

    #[tokio::test]
    async fn test_dns_resolution() {
        use hyper::client::connect::dns::Name;
        use hyper::service::Service;
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
}
