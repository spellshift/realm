# Tavern (Rust GraphQL Client)

## Overview

* This client library is intended to provide a minimal set of types and operations for agents to interact with the Tavern GraphQL API
* GraphQL types are generated using the `graphql-client generate` command, provided by the `graphql_client_cli` crate
  * `codegen.sh` will install and run this command with the correct parameters
  * The Tavern (server implementation) Go generate will automatically populate the graphql schema and run this command when the schema changes
* Mutations and the data requested from the server are present in the `graphql/mutations.graphql` file
* An HTTP transport is included as a default transport (`tavern::http`, feature `http`), however custom implementors of `tavern::Executor` may be defined and used with `tavern::Client`
  * See for example `MockTransport` in our test cases, or look at the `tavern::http::Transport` implementation for more information

## Example Usage

```rust
use tavern::http::{new_client};
use tavern::{ClaimTasksInput, SubmitTaskResultInput, HostPlatform, Task};
use chrono::Utc;

#[tokio::main]
async fn main() {
    let client = new_client("https://mydomain.com/graphql", "supersecret");

    // Fetch new tasks
    let tasks: Vec<Task> = client.claim_tasks(ClaimTasksInput{
        agent_identifier: String::from("example"),
        session_identifier: String::from("123456"),
        host_identifier: String::from("ABCDEFG"),
        principal: String::from("root"),
        hostname: String::from("web"),
        host_platform: HostPlatform::Linux,
        host_primary_ip: Some(String::from("10.0.0.1")),
    }).await.expect("TODO: Handle errors :D");

    // ... execute tasks ...

    // Report output
    client.submit_task_result(SubmitTaskResultInput{
        task_id: String::from("5"),
        exec_started_at: Utc::now(),
        exec_finished_at: Some(Utc::now()),
        output: String::from("It works!"),
        error: None,
    }).await.expect("TODO: Handle errors :D");
}
```
