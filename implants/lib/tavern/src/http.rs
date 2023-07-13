use async_trait::async_trait;
use graphql_client::{QueryBody};
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::{time::Duration};

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
    async fn exec<Variables: Serialize+Send, GraphQLResponse: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<GraphQLResponse> {
        let req: reqwest::RequestBuilder = self.http.post(self.url.as_str())
            .json(&query)
            .header("Content-Type", "application/json")
            .header(AUTH_HEADER, self.auth_token.as_str());
        let resp = req.send().await?;
        let gql_resp = resp.json::<GraphQLResponse>().await?;
        Ok(gql_resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        ClaimTasksInput,
        SubmitTaskResultInput,
        HostPlatform,
    };
    use httptest::{Server, Expectation, matchers::*, responders::*};
    use chrono::Utc;

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

    #[tokio::test]
    async fn test_claim_tasks() {
        let input = ClaimTasksInput {
            session_identifier: "bdf0b788-b32b-4faf-8719-93cd3955b043".to_string(),
            host_identifier: "bdf0b788-b32b-4faf-8719-93cd3955b043".to_string(),
            agent_identifier: "imix".to_string(),
            principal: "root".to_string(),
            hostname: "localhost".to_string(),
            host_platform: HostPlatform::Linux,
            host_primary_ip: Some("10.0.0.1".to_string()),
        };

        let server = Server::run();

        let expected_req = Expectation::matching(request::method_path("POST", "/graphql")).
            respond_with(
                json_encoded(serde_json::json!({
                    "data": {
                        "claimTasks": [
                            {
                                "id":"17179869185",
                                "job": {
                                    "id":"4294967297",
                                    "name":"test_exe3",
                                    "tome": {
                                        "id":"21474836482",
                                        "name":"Shell Execute",
                                        "description":r#"Execute a shell script using the default interpreter. /bin/bash for macos \u0026 linux, and cmd.exe for windows."#,
                                        "paramDefs":"{\"cmd\":\"string\"}","eldritch":"sys.shell(eld.get_param('cmd'))",
                                        "files":[]
                                    },
                                    "bundle":null
                                },
                            },
                            {
                                "id":"17179869186",
                                "job": {
                                    "id":"4294967298",
                                    "name":"test_exe2",
                                    "tome": {
                                        "id":"21474836482",
                                        "name":"Shell Execute",
                                        "description":r#"Execute a shell script using the default interpreter. /bin/bash for macos \u0026 linux, and cmd.exe for windows."#,
                                        "parameters":"{\"cmd\":\"string\"}","eldritch":"sys.shell(eld.get_param('cmd'))",
                                        "files":[]
                                    },
                                    "bundle":null
                                },
                            },
                        ],
                    },
                }))
            );
        server.expect(expected_req);

        let url = server.url("/graphql").to_string();
        let client = new_client(url.as_str(), "super_secret").expect("failed to configure client");
        let response = client.claim_tasks(input).
            await.
            expect("failed to send graphql request");
        for task in response {
            assert!(task.job.name.contains("test_exe"))
        }
    }

    #[tokio::test]
    async fn test_submit_task_result() {
        let start_time = Utc::now();

        let server = Server::run();
        let expected_req = Expectation::matching(request::method_path("POST", "/graphql")).
            respond_with(
                json_encoded(serde_json::json!({
                    "data": {
                        "submitTaskResult": {
                            "id": "17179869186",
                        },
                    },
                }))
            );
        server.expect(expected_req);

        let url = server.url("/graphql").to_string();
        let client = new_client(url.as_str(), "super_secret").expect("failed to configure client");

        let input = SubmitTaskResultInput {
            task_id: "17179869186".to_string(),
            exec_started_at: start_time,
            exec_finished_at: Some(Utc::now()),
            output: "whoami".to_string(),
            error: None,
        };
        client.submit_task_result(input).await.
            expect("failed to submit task result");
    }
}