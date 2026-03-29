use anyhow::Result;
use eldritch_agent::Agent;
use std::sync::Arc;

pub fn tcp(addr: String, agent: Arc<dyn Agent>) -> Result<i64> {
    tokio::spawn(async move {
        if let Err(_e) = crate::tcp_impl::start_tcp_chain_server(&addr, agent).await {
            #[cfg(feature = "verbose-logging")]
            log::error!("tcp chain proxy error: {}", _e);
        }
    });

    Ok(0)
}
