use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct GraphQLClaimTasksInput {
    principal: String,
    hostname: String,
    #[serde(rename="sessionIdentifier")]
    session_identifier: String,
    #[serde(rename="hostIdentifier")]
    host_identifier: String,
    #[serde(rename="agentIdentifier")]
    agent_identifier: String,
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

pub async fn gql_claim_task(){
    let taskinput = GraphQLVariableEnvelope{
        input: GraphQLClaimTasksInput {
            principal: "root".to_string(),
            hostname: "localhost".to_string(),
            session_identifier: "s1234".to_string(),
            host_identifier: "h1234".to_string(),
            agent_identifier: "a1234".to_string(),
        }};
    let req_body = serde_json::to_string(&GraphQLRequestEnvelope {
        operation_name: String::from("ImixCallback"),
        query: String::from(r#"mutation ImixCallback($input: ClaimTasksInput!) { claimTasks(input: $input) { id }}"#),
        variables: taskinput,
    }).unwrap();
    println!("{}", req_body);

    let client = reqwest::Client::new();
    match client.post("http://127.0.0.1:80/graphql")
    .header("Content-Type", "application/json")
    .body(req_body)
    .send()
    .await {
        Ok(http_response) => {
            match http_response.text().await {
                Ok(text_recieved) => println!("Okay.\n{:?}", text_recieved),
                Err(text_error) => println!("Error.\n{:?}", text_error),
            }
        },
        Err(http_error) => {
            println!("Error.\n{:?}", http_error);
        },
    }
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
            gql_claim_task()
        );
        println!("{:?}", response)
        
    }
}
