mod mutations;
mod scalars;

// Only export http if it has been enabled.
#[cfg(feature = "http")]
pub mod http;

// Re-export relevant GraphQL types with more accessible names.
pub use mutations::claim_tasks::{
    ClaimTasksInput,
    ClaimTasksClaimTasks as Task,
    ClaimTasksClaimTasksQuest as Quest,
    ClaimTasksClaimTasksQuestTome as Tome,
    ClaimTasksClaimTasksQuestTomeFiles as File,
    ClaimTasksClaimTasksQuestBundle as Bundle,
    BeaconHostPlatform as HostPlatform,
    ResponseData as ClaimTasksResponseData,
};
pub use mutations::submit_task_result::{
    SubmitTaskResultInput,
    SubmitTaskResultSubmitTaskResult as SubmitTaskResult,
    ResponseData as SubmitTaskResultResponseData,
};

pub use graphql_client::{
    Response as GraphQLResponse,
};

use async_trait::async_trait;
use graphql_client::{GraphQLQuery, QueryBody, Response};
use anyhow::{anyhow, Error, Result};
use serde::{Serialize, de::DeserializeOwned};

/*
 * An Executor is responsible for serializing a GraphQL query, sending the query to a server, and deserializing a result.
 */
#[async_trait]
pub trait Executor {
    async fn exec<Variables: Serialize+Send, GraphQLResponse: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<GraphQLResponse>;
}

/*
 * Client provides a convinient interface for calling relevant Tavern GraphQL mutations using the underlying transport (e.g. HTTP).
 */
pub struct Client<E: Executor> {
    transport: E
}

impl<E: Executor> Client<E> {
    /*
     * Fetches new tasks for the agent to execute, if any are available.
     */
    pub async fn claim_tasks(&self, input: ClaimTasksInput) -> Result<Vec<Task>> {
        let vars = mutations::claim_tasks::Variables{input};
        let query = mutations::ClaimTasks::build_query(vars);
        let resp = self.exec::<mutations::claim_tasks::Variables, mutations::claim_tasks::ResponseData>(query).await?;
        Ok(resp.claim_tasks)
    }

    pub async fn submit_task_result(&self, input: SubmitTaskResultInput) -> Result<()> {
        let vars = mutations::submit_task_result::Variables{input};
        let query: QueryBody<mutations::submit_task_result::Variables> = mutations::SubmitTaskResult::build_query(vars);
        self.exec::<mutations::submit_task_result::Variables, mutations::submit_task_result::ResponseData>(query).await?;
        Ok(())
    }

    // Wraps transport calls with error handling.
    async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<ResponseData> {
        let resp: Response<ResponseData> = self.transport.exec(query).await?;

        if let Some(errors) = resp.errors {
            if errors.len() > 0 {
                return Err(join_errors(errors));
            }
        }
        match resp.data {
            Some(data) => return Ok(data),
            None =>  return Err(anyhow!("no data returned from api")),
        }
    }
}

// Combine multiple GraphQL errors into a single Error to be returned.
fn join_errors(errors: Vec<graphql_client::Error>) -> Error {
    Error::msg(
        errors.iter().
            map(|err| err.to_string()).
            collect::<Vec<String>>().
            join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use serde::{Serialize};
    use crate::mutations::submit_task_result::SubmitTaskResultSubmitTaskResult;

    // Defines a MockTransport which simply returns the expected response.
    struct MockTransport {
        expected_response: String,
    }

    impl MockTransport {
        pub fn get_client<T: Serialize>(expected_response: T) -> Client<MockTransport> {
            let expected_data = serde_json::to_string(&expected_response).expect("failed to serialize expected response");

            Client{
                transport: MockTransport{
                    expected_response: expected_data,
                }
            }
        }
    }

    #[async_trait]
    impl Executor for MockTransport {
        async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, _query: QueryBody<Variables>) -> Result<ResponseData> {
            match serde_json::from_str(self.expected_response.as_str()) {
                Ok(resp) => return Ok(resp),
                Err(error) => return Err(anyhow!("failed to parse expected response: {}", error)),
            }
        }
    }

    #[tokio::test]
    async fn claim_tasks() {
        let expected_resp = Response{
            data: Some(mutations::claim_tasks::ResponseData{
                claim_tasks: vec![
                    Task{
                        id: String::from("5"),
                        quest: Quest{
                            id: String::from("10"),
                            name: String::from("test_quest"),
                            parameters: None,
                            tome: Tome{
                                id: String::from("15"),
                                name: String::from("test_tome"),
                                description: String::from("used in a unit test :)"),
                                eldritch: String::from(r#"print("hello world!")"#),
                                param_defs: None,
                                files: None,
                            },
                            bundle: None,
                        }
                    }
              ],
            }),
            errors: None,
            extensions: None,
        };
        let client = MockTransport::get_client(expected_resp);
        let input = ClaimTasksInput{
            principal: String::from("test"),
            hostname: String::from("test"),
            host_platform: HostPlatform::Windows,
            host_primary_ip: Some(String::from("test")),
            beacon_identifier: String::from("test"),
            host_identifier: String::from("test"),
            agent_identifier: String::from("test"),
        };
        let tasks: Vec<Task> = client.claim_tasks(input).await.expect("failed to claim tasks");
        assert!(tasks.len() == 1);
        assert!(tasks[0].id == "5");
    }

    #[tokio::test]
    async fn submit_task_result() {
        let expected_resp = Response{
            data: Some(mutations::submit_task_result::ResponseData{
                submit_task_result: Some(SubmitTaskResultSubmitTaskResult{
                    id: String::from("5"),
                }),
            }),
            errors: None,
            extensions: None,
        };
        let client = MockTransport::get_client(expected_resp);
        let input = SubmitTaskResultInput{
            task_id: String::from("5"),
            exec_started_at: Utc::now(),
            exec_finished_at: Some(Utc::now()),
            output: String::from("It works!"),
            error: None,
        };
        println!("task_response: {}", serde_json::to_string(&input).unwrap());
        let resp = client.submit_task_result(input).await;
        assert!(resp.is_ok());
    }
}