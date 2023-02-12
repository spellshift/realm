use anyhow::{Error};
use serde::{Serialize, Deserialize};

// ------------- GraphQL claimTasks request -------------
#[derive(Serialize, Deserialize)]
pub(crate) struct GraphQLClaimTasksInput {
    pub(crate) principal: String,
    pub(crate) hostname: String,
    #[serde(rename="sessionIdentifier")]
    pub(crate) session_identifier: String,
    #[serde(rename="hostIdentifier")]
    pub(crate) host_identifier: String,
    #[serde(rename="agentIdentifier")]
    pub(crate) agent_identifier: String,
}

#[derive(Serialize, Deserialize)]
struct GraphQLVariableEnvelope {
    input: GraphQLClaimTasksInput,
}

#[derive(Serialize, Deserialize)]
struct GraphQLRequestEnvelope {
    query: String,
    variables: GraphQLVariableEnvelope,
    #[serde(rename="operationName")]
    operation_name: String,
}

// ------------- GraphQL claimTasks response -------------

#[derive(Serialize, Deserialize)]
pub struct GraphQLTome {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: Option<String>,
    pub eldritch: String,
    pub files: Vec<GraphQLFile>
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLFile {
    pub id: String,
    pub name: String,
    pub size: u32,
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLJob {
    pub id: String,
    pub name: String,
    pub tome: GraphQLTome,
    pub bundle: Option<GraphQLFile>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLTask {
    pub id: String,
    pub job: GraphQLJob
}

#[derive(Serialize, Deserialize)]
struct GraphQLClaimTasksOutput {
    #[serde(rename="claimTasks")]
    claim_tasks: Vec<GraphQLTask>,
}

#[derive(Serialize, Deserialize)]
struct GraphQLResponseEnvelope {
    data: GraphQLClaimTasksOutput,
}


pub async fn gql_claim_tasks(uri: String) -> Result<Vec<GraphQLTask>, Error> {
    let input_variable = GraphQLClaimTasksInput {
        principal: "root".to_string(),
        hostname: "localhost".to_string(),
        session_identifier: "s1234".to_string(),
        host_identifier: "h1234".to_string(),
        agent_identifier: "a1234".to_string(),
    };

    let req_body = match serde_json::to_string(&
    GraphQLRequestEnvelope {
        operation_name: String::from("ImixCallback"),
        query: String::from(r#"
mutation ImixCallback($input: ClaimTasksInput!) {
    claimTasks(input: $input) { 
        id,
        job { 
            id,
            name,
            tome {
                id,
                name,
                description,
                parameters,
                eldritch,
                files {
                    id,
                    name,
                    size,
                    hash,
                }
            },
            bundle {
                id,
                name,
                size,
                hash,
            }
        }
    }
}"#),
        variables: GraphQLVariableEnvelope{
            input: input_variable
        },
    }) {
        Ok(json_req_body) => json_req_body,
        Err(error) => return Err(anyhow::anyhow!("Failed encode request to JSON\n{}", error)),
    };

    // println!("{}", req_body);
    let client = reqwest::Client::new();

    let response_text = match client.post(uri)
    .header("Content-Type", "application/json")
    .body(req_body)
    .send()
    .await {
        Ok(http_response) => {
            match http_response.text().await {
                Ok(text_recieved) => text_recieved,
                Err(text_error) => return Err(anyhow::anyhow!("Error decoding http response.\n{}", text_error)),
            }
        },
        Err(http_error) => return Err(anyhow::anyhow!("Error making http request.\n{}", http_error)),
    };

    let graphql_response: GraphQLResponseEnvelope = match serde_json::from_str(&response_text) {
        Ok(new_tasks_object) => new_tasks_object,
        Err(error) => return Err(anyhow::anyhow!("Error deserializing GraphQL response.\n{}", error)),
    };
    let new_tasks = graphql_response.data.claim_tasks;
    Ok(new_tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imix_graphql_claim_task_test_standard() {
        println!("HERE");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            gql_claim_tasks("http://127.0.0.1:80/graphql".to_string())
        );
        for task in response.unwrap() {
            println!("{}", serde_json::to_string(&task).unwrap())
        }
        
    }
}
