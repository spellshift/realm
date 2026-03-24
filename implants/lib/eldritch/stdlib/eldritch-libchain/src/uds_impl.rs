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
    log::debug!("[chain-server] incoming forward request: {}", path);

    let grpc_response = |body: Vec<u8>| {
        HyperResponse::builder()
            .status(200)
            .header("content-type", "application/grpc+proto")
            .header("grpc-status", "0")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    };

    let _grpc_error = |code: u32, msg: String| {
        log::error!("chain proxy: {} — {}", path, msg);
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
            log::error!("forward_raw error: {}", e);
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

/// Start a chain proxy server on the given UDS path.
///
/// # Message Forwarding
///
/// This function implements Agent A's side of the chained transport:
///
/// 1. **Agent B** (the inner, chained agent) calls `chain.uds(path)` in an Eldritch script.
///    That spawns this function as an async task.
/// 2. **Agent A** (this process) connects outward to the UDS socket that Agent B has already
///    bound and is listening on.
/// 3. Agent A then serves HTTP/2 on that connection. From Agent B's perspective it is making
///    gRPC calls to a local server; from Agent A's perspective it is a gRPC server accepting
///    those calls.
/// 4. When a gRPC call arrives (e.g. `ClaimTasks`, `ReportOutput`), [`handle_request`] decodes
///    the protobuf body and dispatches to the corresponding [`Agent`] trait method on `agent`.
/// 5. Those [`Agent`] trait methods are backed by **Agent A's own upstream transport** (typically
///    mTLS gRPC to Tavern). Responses flow back through the same UDS connection to Agent B.
///
/// In short: Agent B's C2 messages enter through the UDS socket and exit through Agent A's
/// normal upstream channel — Agent A acts as a transparent proxy.
pub async fn start_chain_server(path: &str, agent: Arc<dyn Agent>) -> Result<()> {
    log::info!(
        "[chain-server] connecting to Agent B's UDS socket at {}",
        path
    );

    log::debug!("[chain-server] attempting UnixStream::connect({})", path);
    let stream = match tokio::net::UnixStream::connect(path).await {
        Ok(s) => {
            log::info!("[chain-server] connected to Agent B at {}", path);
            s
        }
        Err(e) => {
            log::error!("[chain-server] connect error: {}", e);
            return Err(anyhow!(
                "Failed to connect to Agent B UDS socket at {}: {}",
                path,
                e
            ));
        }
    };

    let agent_clone = agent.clone();
    let io = TokioIo::new(stream);

    log::debug!("[chain-server] starting HTTP/2 on connection to Agent B");
    let service = service_fn(move |req| {
        let a = agent_clone.clone();
        async move { handle_request(a, req).await }
    });

    if let Err(err) = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
        .serve_connection(io, service)
        .await
    {
        log::error!("[chain-server] HTTP/2 connection error: {:?}", err);
        return Err(anyhow!("HTTP/2 connection to Agent B failed: {:?}", err));
    }

    log::info!("[chain-server] connection closed cleanly");
    Ok(())
}
