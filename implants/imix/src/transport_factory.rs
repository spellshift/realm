use anyhow::{anyhow, Result};
use pb::c2::transport::Type as TransportType;
use pb::config::Config;
use transport::Transport;

pub fn create_transport(config: Config) -> Result<Box<dyn Transport + Send + Sync>> {
    let transport_type = config
        .info
        .as_ref()
        .and_then(|info| info.available_transports.as_ref())
        .and_then(|at| {
            let active_idx = at.active_index as usize;
            at.transports
                .get(active_idx)
                .or_else(|| at.transports.first())
        })
        .map(|t| t.r#type)
        .ok_or_else(|| anyhow!("No transports configured"))?;

    match TransportType::try_from(transport_type) {
        Ok(TransportType::TransportUds) => {
            #[cfg(feature = "uds")]
            return Ok(Box::new(
                pro_transports::uds::UdsTransport::new(config)?,
            ));
            #[cfg(not(feature = "uds"))]
            return Err(anyhow!("UDS transport not enabled (build with --features uds)"));
        }
        _ => transport::create_transport(config),
    }
}
