mod mutations;
mod scalars;

#[cfg(feature = "http")]
pub mod http;

pub use mutations::claim_tasks::{
    ClaimTasksInput,
    ClaimTasksClaimTasks as Task,
    ClaimTasksClaimTasksJob as Job,
    ClaimTasksClaimTasksJobTome as Tome,
    ClaimTasksClaimTasksJobTomeFiles as File,
    ClaimTasksClaimTasksJobBundle as Bundle,
    SessionHostPlatform as HostPlatform,
};

use async_trait::async_trait;
use graphql_client::{GraphQLQuery, QueryBody, Response};
use anyhow::{anyhow, Error, Result};
use serde::{Serialize, de::DeserializeOwned};


#[async_trait]
pub trait Executor {
    async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<ResponseData>;
}

pub struct Client<E: Executor> {
    transport: E
}

impl<E: Executor> Client<E> {
    pub async fn claim_tasks(&self, input: ClaimTasksInput) -> Result<Vec<Task>> {
        let vars = mutations::claim_tasks::Variables{
            input: input,
        };
        let query: QueryBody<mutations::claim_tasks::Variables> = mutations::ClaimTasks::build_query(vars);
        let resp: mutations::claim_tasks::ResponseData = self.exec::<mutations::claim_tasks::Variables, mutations::claim_tasks::ResponseData>(query).await?;
        Ok(resp.claim_tasks)
    }


    async fn exec<Variables: Serialize+Send, ResponseData: DeserializeOwned>(&self, query: QueryBody<Variables>) -> Result<ResponseData> {
        let resp: Response<ResponseData> = self.transport.exec(query).await?;

        if let Some(errors) = resp.errors {
            return Err(join_errors(errors));
        }
        match resp.data {
            Some(data) => return Ok(data),
            None =>  return Err(anyhow!("no data returned from api")),
        }
    }
}

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
    use serde::{Serialize};

    use super::*;

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
                        job: Job{
                            id: String::from("10"),
                            name: String::from("test_job"),
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
            session_identifier: String::from("test"),
            host_identifier: String::from("test"),
            agent_identifier: String::from("test"),
        };
        let tasks: Vec<Task> = client.claim_tasks(input).await.expect("failed to claim tasks");
        assert!(tasks.len() == 1);
        assert!(tasks[0].id == "5");
    }
}