#![cfg(feature = "http1")]

use anyhow::Result;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use pb::c2::{ClaimTasksRequest, ClaimTasksResponse};
use prost::Message;
use std::convert::Infallible;
use std::net::SocketAddr;
use transport::{Http1, Transport};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    match path {
        "/c2.C2/ClaimTasks" => {
            let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let _claim_req = ClaimTasksRequest::decode(&body[..]).unwrap();
            let resp = ClaimTasksResponse::default();
            let body = Body::from(resp.encode_to_vec());
            Ok(Response::new(body))
        }
        _ => Ok(Response::builder().status(404).body(Body::empty()).unwrap()),
    }
}

async fn mock_server() -> Result<SocketAddr> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let server = Server::bind(&addr).serve(make_svc);
    let addr = server.local_addr();
    tokio::spawn(server);
    Ok(addr)
}

#[tokio::test]
async fn test_claim_tasks() -> Result<()> {
    let addr = mock_server().await?;
    let uri = format!("http://{}", addr);
    let mut transport = Http1::new(uri, None)?;

    let request = ClaimTasksRequest::default();
    let response = transport.claim_tasks(request).await?;

    assert_eq!(response, ClaimTasksResponse::default());

    Ok(())
}
