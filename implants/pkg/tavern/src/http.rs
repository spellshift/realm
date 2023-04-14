use async_trait::async_trait;
use graphql_client::{QueryBody, Response};
use anyhow::{anyhow, Result};
use serde::{Serialize, de::DeserializeOwned};

const DEFAULT_TIMEOUT: i32 = 5;

/*
 * Prepares a new client that will provide an interface to Tavern's GraphQL HTTP API at the provided url.
 * Advanced HTTP configuration can be supplied by modifying the client.transport.http field.
 */
pub fn new_client(url: String) -> crate::Client<Executor> {
    crate::Client {
        transport: Transport::new(url),
    }
}

/*
 * Transport is responsible for serializing queries, making http requests using the configured client, and deserializing API responses.
 */
pub struct Transport {
    pub url: String,
    pub http: reqwest::Client,
}


impl Transport {
    /*
     * Prepares a new default HTTP transport using the provided API endpoint.
     */
    pub fn new(url: String) -> Self {
        Transport {
            url,
            http: reqwest::Client::builder()
                    .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
                    .danger_accept_invalid_certs(true)
                    .build(),
        }
    }
}

/*
 * Implement the Executor trait for Transport so that it may be used by the GraphQL library.
 */
#[async_trait]
impl crate::Executor for Transport {
    async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<ResponseData> {
        let req = self.http.post(self.url.as_str()).json(&query).send();
        let resp = req.await?;
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
    fn new_transport() {
        let transport = Transport::new(String::from("https://google.com/"));
        assert!(transport.url == "https://google.com/");
    }

    #[test]
    fn new_client() {
        let client = new_client("https://google.com/");
        assert!(client.transport.url = "https://google.com/");
    }
}