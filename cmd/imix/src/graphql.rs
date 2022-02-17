use serde::{Serialize, Deserialize};
use std::time::Duration;

use hyper::{Client, Request, Method, Body};
use hyper::body;

use hyper_tls::HttpsConnector;
use hyper_timeout::TimeoutConnector;

#[derive(Serialize, Deserialize)]
struct GraphQLRequest {
    #[serde(rename="operationName")]
    operation_name: String,
    query: String,
    variables: String
}

#[derive(Serialize, Deserialize)]
struct GraphQLCallbackResponse {
    id: u64
}

#[derive(Serialize, Deserialize)]
struct GraphQLMutationsResponse {
    callback: GraphQLCallbackResponse
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLResponse {
    data: GraphQLMutationsResponse
}

pub async fn call(variables: String, uri: String, timeout: u64) -> Result<GraphQLResponse, super::Error>{
    let h = HttpsConnector::new();
    let mut connector = TimeoutConnector::new(h);
    connector.set_connect_timeout(Some(Duration::from_secs(timeout)));
    connector.set_read_timeout(Some(Duration::from_secs(timeout)));
    connector.set_write_timeout(Some(Duration::from_secs(timeout)));
    let client = Client::builder().build::<_, hyper::Body>(connector);

    let req_body = serde_json::to_string(&GraphQLRequest {
        operation_name: String::from("ImixCallback"),
        query: String::from(r#"
        mutation ImixCallback($target_id: ID!) {
            callback(input: $target_id) {
                id
            }
        }"#),
        variables: variables,
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("X-Realm-Auth", "letmeinnn")
        .body(Body::from(req_body))?;

    let http_resp = client.request(req).await?;
    let http_resp_body = body::to_bytes(http_resp).await?;
    let resp: GraphQLResponse = serde_json::from_slice(&http_resp_body)?;
    
    // TODO: handles API errors also
    Ok(resp)
}