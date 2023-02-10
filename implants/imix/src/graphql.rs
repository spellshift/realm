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



pub async fn gql_claim_task(){
    let data = GraphQLClaimTasksInput{
        principal: "root".to_string(),
        hostname: "localhost".to_string(),
        session_identifier: "s1234".to_string(),
        host_identifier: "h1234".to_string(),
        agent_identifier: "a1234".to_string(),
    };
    let client = reqwest::Client::new();
    match client.post("http://27.0.0.1:80/")
    .json(&data)
    .send()
    .await {
        Ok(response) => {
            println!("Okay.\n{:?}", response);
        },
        Err(error) => {
            println!("Error.\n{:?}", error);
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
