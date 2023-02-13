use anyhow::{Error};
use serde::{Serialize, Deserialize};
// https://time-rs.github.io/api/time/format_description/well_known/struct.Rfc3339.html
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

    let client = reqwest::Client::new();

    let response_text = match client.post(uri)
    .header("Content-Type", "application/json")
    .header("X-Realm-Auth", "letmeinnn")
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


// ------------- GraphQL SubmitTaskResultInput request -------------

#[derive(Serialize, Deserialize)]
pub struct SubmitTaskResultInput {
    #[serde(rename="taskID")]
    pub task_id: String,
    #[serde(rename="execFinishedAt")]
    pub exec_started_at: String,
    #[serde(rename="execFinishedAt")]
    pub exec_finished_at: Option<String>,
    pub output: String,
    pub error: String,
}

// pub async fn gql_post_task_result(uri: String, task_res: SubmitTaskResultInput) -> Result<Vec<GraphQLTask>, Error> {
// /*
// {
//   "query": "mutation ImixPostResult($input: SubmitTaskResultInput!) { submitTaskResult(input: $input) { \n\tid\n}}",
//   "variables": {
//     "input": {
//       "taskID": "17179869186",
//       "execStartedAt": "1985-04-12T23:20:50.52Z",
//       "execFinishedAt": "1995-04-12T23:20:50.52Z",
//       "output": "root!",
//       "error": ""
//     }
//   },
//   "operationName": "ImixPostResult"
// }
// */
//     let req_body = match serde_json::to_string(&
//         GraphQLRequestEnvelope {
//             operation_name: String::from("ImixCallback"),
//             query: String::from(r#"
//     mutation ImixPostResult($input: ClaimTasksInput!) {
//         submitTaskResult(input: $input) { 
//             id
//         }
//     }"#),
//         variables: GraphQLVariableEnvelope{
//             input: task_res
//         },
//     }) {
//         Ok(json_req_body) => json_req_body,
//         Err(error) => return Err(anyhow::anyhow!("Failed encode request to JSON\n{}", error)),
//     };

//     let client = reqwest::Client::new();

//     let response_text = match client.post(uri)
//     .header("Content-Type", "application/json")
//     .header("X-Realm-Auth", "letmeinnn")
//     .body(req_body)
//     .send()
//     .await {
//         Ok(http_response) => {
//             match http_response.text().await {
//                 Ok(text_recieved) => text_recieved,
//                 Err(text_error) => return Err(anyhow::anyhow!("Error decoding http response.\n{}", text_error)),
//             }
//         },
//         Err(http_error) => return Err(anyhow::anyhow!("Error making http request.\n{}", http_error)),
//     };

//     let graphql_response: GraphQLResponseEnvelope = match serde_json::from_str(&response_text) {
//         Ok(new_tasks_object) => new_tasks_object,
//         Err(error) => return Err(anyhow::anyhow!("Error deserializing GraphQL response.\n{}", error)),
//     };
//     let new_tasks = graphql_response.data.claim_tasks;
//     Ok(new_tasks)
// }




#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};

    #[test]
    fn imix_graphql_claim_task_test_standard() {
        let server = Server::run();
        server.expect(Expectation::matching(request::method_path("POST", "/graphql"))
            .respond_with(status_code(200).body(r#"{"data":{"claimTasks":[{"id":"17179869185","job":{"id":"4294967297","name":"test_exe3","tome":{"id":"21474836482","name":"Shell Execute","description":"Execute a shell script using the default interpreter. /bin/bash for macos \u0026 linux, and cmd.exe for windows.","parameters":"{\"cmd\":\"string\"}","eldritch":"sys.shell(eld.get_param('cmd'))","files":[]},"bundle":null}},{"id":"17179869186","job":{"id":"4294967298","name":"test_exe2","tome":{"id":"21474836482","name":"Shell Execute","description":"Execute a shell script using the default interpreter. /bin/bash for macos \u0026 linux, and cmd.exe for windows.","parameters":"{\"cmd\":\"string\"}","eldritch":"sys.shell(eld.get_param('cmd'))","files":[]},"bundle":null}}]}}"#))
        );

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            gql_claim_tasks(server.url("/graphql").to_string())
        ).unwrap();
        for task in response {
            assert!(task.job.name.contains("test_exe"))
        }
    }
}

/*
iQL script
# Create tome
mutation CreateTome ($input: CreateTomeInput!) {
  createTome(input: $input) {
    id
  }
}

{
  "input": {
    "name": "Shell Execute",
    "description": "Execute a shell script using the default interpreter. /bin/bash for macos & linux, and cmd.exe for windows.",
    "parameters": "{\"cmd\":\"string\"}",
    "eldritch": "sys.shell(eld.get_param('cmd'))",
    "fileIDs": []
  }
}

# Get session IDs
query get_sessions {
	sessions {
    id
    identifier
  }
}

# Queue Job with tome and session


*/
