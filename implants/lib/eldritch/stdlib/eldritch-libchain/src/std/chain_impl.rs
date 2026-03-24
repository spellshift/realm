use anyhow::Result;
use eldritch_agent::Agent;
use std::sync::Arc;

pub fn uds(path: String, agent: Arc<dyn Agent>) -> Result<i64> {
    tokio::spawn(async move {
        if let Err(e) = crate::uds_impl::start_chain_server(&path, agent).await {
            log::error!("chain proxy error: {}", e);
        }
    });

    Ok(0)
}

pub fn tcp(addr: String, agent: Arc<dyn Agent>) -> Result<i64> {
    tokio::spawn(async move {
        if let Err(e) = crate::tcp_impl::start_tcp_chain_server(&addr, agent).await {
            log::error!("tcp chain proxy error: {}", e);
        }
    });

    Ok(0)
}
