use async_trait::async_trait;
use graphql_client::{QueryBody, Response};
use anyhow::{anyhow, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

const DEFAULT_TIMEOUT: u64 = 5;
const AUTH_HEADER: &str = "X-Realm-Auth";

/*
 * Prepares a new GraphQL client that will provide an interface to Tavern's HTTP GraphQL API at the provided url.
 * Advanced HTTP configuration can be supplied by modifying the client.transport.http field.
 */
pub fn new_client(url: &str, auth_token: &str) -> Result<crate::Client<Transport>> {
    let transport = Transport::new(url, auth_token)?;
    Ok(crate::Client {transport})
}

/*
 * Transport is responsible for serializing queries, making http requests using the configured client, and deserializing API responses.
 */
pub struct Transport {
    pub auth_token: String,
    pub url: String,
    pub http: reqwest::Client,
}


impl Transport {
    /*
     * Prepares a new default HTTP transport using the provided API endpoint.
     */
    pub fn new(url: &str, auth_token: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
                        .danger_accept_invalid_certs(true)
                        .build()?;
        Ok(Transport {
            auth_token: String::from(auth_token),
            url: String::from(url),
            http: client,
        })
    }
}

/*
 * Implement the Executor trait for Transport so that it may be used by the GraphQL library.
 */
#[async_trait]
impl crate::Executor for Transport {
    async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<ResponseData> {
        let req = self.http.post(self.url.as_str())
            .json(&query)
            .header("Content-Type", "application/json")
            .header(AUTH_HEADER, self.auth_token.as_str());
        let resp = req.send().await?;
        let body = resp.json::<Response<ResponseData>>().await?;

        match body.data {
            Some(data) => return Ok(data),
            None => return Err(anyhow!("no data returned from HTTP transport")),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transport() {
        let expected_url = "https://google.com/";
        let expected_auth = "auth_test";

        let transport = Transport::new(expected_url, expected_auth).expect("failed to initialize transport");
        assert_eq!(expected_url, transport.url);
        assert_eq!(expected_auth, transport.auth_token);
    }

    #[test]
    fn test_new_client() {
        let expected_url = "https://google.com/";
        let expected_auth = "auth_test";

        let client = new_client(expected_url, expected_auth).expect("failed to initialize client");
        assert_eq!(expected_url, client.transport.url );
        assert_eq!(expected_auth, client.transport.auth_token);
    }
}