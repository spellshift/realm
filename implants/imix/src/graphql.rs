use std::time::Duration;

use anyhow::{Error};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
// https://time-rs.github.io/api/time/format_description/well_known/struct.Rfc3339.html
// ------------- GraphQL claimTasks request -------------
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphQLClaimTasksInput {
    pub principal: String,
    pub hostname: String,
    #[serde(rename="sessionIdentifier")]
    pub session_identifier: String,
    #[serde(rename="hostIdentifier")]
    pub host_identifier: String,
    #[serde(rename="hostPlatform")]
    pub host_platform: String,
    #[serde(rename="agentIdentifier")]
    pub agent_identifier: String,
}

#[derive(Serialize, Deserialize)]
struct GraphQLClaimTaskVariableEnvelope {
    input: GraphQLClaimTasksInput
}

#[derive(Serialize, Deserialize)]
struct GraphQLClaimRequestEnvelope {
    query: String,
    variables: GraphQLClaimTaskVariableEnvelope,
    #[serde(rename="operationName")]
    operation_name: String,
}

// ------------- GraphQL claimTasks response -------------

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphQLTome{
    pub id: String, 
    pub name: String, 
    pub description: String, 
    #[serde(rename="paramDefs")]
    pub param_defs: Option<String>, 
    pub eldritch: String,
    pub files: Vec<GraphQLFile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphQLFile {
    pub id: String,
    pub name: String,
    pub size: u32,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphQLJob {
    pub id: String,
    pub name: String,
    pub tome: GraphQLTome,
    pub parameters: Option<String>,
    pub bundle: Option<GraphQLFile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphQLTask {
    pub id: String,
    pub job: Option<GraphQLJob>
}

#[derive(Serialize, Deserialize)]
struct GraphQLClaimTasksOutput {
    #[serde(rename="claimTasks")]
    claim_tasks: Vec<GraphQLTask>,
}

#[derive(Serialize, Deserialize)]
struct GraphQLClaimTaskResponseEnvelope {
    data: GraphQLClaimTasksOutput,
}

pub async fn gql_claim_tasks(uri: String, claim_task_input_variable: GraphQLClaimTasksInput) -> Result<Vec<GraphQLTask>, Error> {
    let req_body = match serde_json::to_string(&
    GraphQLClaimRequestEnvelope {
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
                paramDefs,
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
        variables: GraphQLClaimTaskVariableEnvelope{
            input: claim_task_input_variable
        },
    }) {
        Ok(json_req_body) => json_req_body,
        Err(error) => return Err(anyhow::anyhow!("Failed encode request to JSON\n{}", error)),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()?;

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

    let graphql_response: GraphQLClaimTaskResponseEnvelope = match serde_json::from_str(&response_text) {
        Ok(new_tasks_object) => new_tasks_object,
        Err(error) => return Err(anyhow::anyhow!("Error deserializing GraphQL response.\n{}\n{}", error, response_text)),
    };
    let new_tasks = graphql_response.data.claim_tasks;
    Ok(new_tasks)
}



// ------------- GraphQL SubmitTaskResultInput request -------------
#[derive(Serialize, Deserialize)]
pub struct GraphQLSubmitTaskResultInput {
    #[serde(rename="taskID")]
    pub task_id: String,
    #[serde(rename="execStartedAt")]
    pub exec_started_at: DateTime<Utc>,
    #[serde(rename="execFinishedAt")]
    pub exec_finished_at: Option<DateTime<Utc>>,
    pub output: String,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLSubmitTaskVariableEnvelope {
    pub input: GraphQLSubmitTaskResultInput
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLSubmitTaskRequestEnvelope {
    pub query: String,
    pub variables: GraphQLSubmitTaskVariableEnvelope,
    #[serde(rename="operationName")]
    pub operation_name: String,
}

// ------------- GraphQL submitTask response -------------

#[derive(Serialize, Deserialize)]
struct GraphQLSubmitTasksOutput {
    #[serde(rename="submitTaskResult")]
    submit_task_result: GraphQLTask,
}

#[derive(Serialize, Deserialize)]
struct GraphQLSubmitTaskResponseEnvelope {
    data: GraphQLSubmitTasksOutput,
}


pub async fn gql_post_task_result(uri: String, task_result: GraphQLSubmitTaskResultInput) -> Result<GraphQLTask, Error> {
    let req_body = match serde_json::to_string(&
        GraphQLSubmitTaskRequestEnvelope {
            operation_name: String::from("ImixPostResult"),
            query: String::from(r#"
    mutation ImixPostResult($input: SubmitTaskResultInput!) {
        submitTaskResult(input: $input) { 
            id
        }
    }"#),
        variables: GraphQLSubmitTaskVariableEnvelope{
            input: task_result
        },
    }) {
        Ok(json_req_body) => json_req_body,
        Err(error) => return Err(anyhow::anyhow!("Failed encode request to JSON\n{}", error)),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()?;

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

    let graphql_response: GraphQLSubmitTaskResponseEnvelope = match serde_json::from_str(&response_text) {
        Ok(new_tasks_object) => new_tasks_object,
        Err(error) => return Err(anyhow::anyhow!("Error deserializing GraphQL response.\n{}", error)),
    };
    let new_tasks = graphql_response.data.submit_task_result;
    Ok(new_tasks)
}




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

        let input_variable = GraphQLClaimTasksInput {
            principal: "root".to_string(),
            hostname: "localhost".to_string(),
            session_identifier: "bdf0b788-b32b-4faf-8719-93cd3955b043".to_string(),
            host_platform: "Linux".to_string(),
            host_identifier: "bdf0b788-b32b-4faf-8719-93cd3955b043".to_string(),
            agent_identifier: "imix".to_string(),
        };
    

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let response = runtime.block_on(
            gql_claim_tasks(server.url("/graphql").to_string(), input_variable)
        ).unwrap();
        for task in response {
            assert!(task.job.unwrap().name.contains("test_exe"))
        }
    }
    #[test]
    fn imix_graphql_post_task_output_standard() {
        let start_time = Utc::now();

        let server = Server::run();
        server.expect(Expectation::matching(request::method_path("POST", "/graphql"))
            .respond_with(status_code(200).body(r#"{"data":{"submitTaskResult":{"id":"17179869186"}}}"#))
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let test_task_response = GraphQLSubmitTaskResultInput {
            task_id: "17179869186".to_string(),
            exec_started_at: start_time,
            exec_finished_at: Some(Utc::now()),
            output: "whoami".to_string(),
            error: "".to_string(),
        };
        let response = runtime.block_on(
            gql_post_task_result(server.url("/graphql").to_string(), test_task_response)
        ).unwrap();

        assert_eq!(response.id,"17179869186".to_string());
    }
    // #[test] // This works.
    // fn imix_graphql_ssl_test() {
    //     let start_time = Utc::now();

    //     let runtime = tokio::runtime::Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap();

    //     let response = runtime.block_on(
    //     {
    //             let client = reqwest::Client::builder()
    //             .timeout(Duration::from_secs(5))
    //             .danger_accept_invalid_certs(true)
    //             .build().unwrap();
        
    //             client.get("https://google.com/")
    //             .header("Content-Type", "application/json")
    //             .header("X-Realm-Auth", "letmeinnn")
    //             .body("")
    //             .send()
    //         }
    //     ).unwrap();

    //     assert_eq!(response.status(), reqwest::StatusCode::OK);
    // }

}

/*
# iQL script
## Create tome
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

## Get session IDs
query get_sessions {
	sessions {
    id
    identifier
  }
}

## imixCallback
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
}

{
  "input": {
        "principal": "root",
        "hostname": "localhost",
        "sessionIdentifier": "s1234",
        "hostIdentifier": "h1234",
        "agentIdentifier": "a1234"
    }
}

## Queue Job with tome and session
mutation createJob($input: CreateJobInput!, $sess:[ID!]!){
  createJob(input: $input, sessionIDs: $sess) {
    id
  }
}

{
  "input": {
    "name": "test_exe1",
    "params": "{}",
    "tomeID": "21474836482"
  },
  "sess": ["8589934593"]
}

## Post task results
mutation postTaskResults($input: SubmitTaskResultInput!) {
  submitTaskResult(input: $input) {
    id
  }
}

{
  "input": {
    "taskID": "17179869186",
    "execStartedAt": "1985-04-12T23:20:50.52Z",
    "execFinishedAt": "1985-04-12T23:40:50.52Z",
    "output": "root",
    "error": ""
  }
}


*/
