#![allow(clippy::all, warnings)]
pub struct ClaimTasks;
pub mod claim_tasks {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "ClaimTasks";
    pub const QUERY : & str = "mutation ClaimTasks($input: ClaimTasksInput!) {\n    claimTasks(input: $input) {\n        id,\n        job {\n            id,\n            name,\n            parameters,\n            tome {\n                id,\n                name,\n                description,\n                paramDefs,\n                eldritch,\n                files {\n                    id,\n                    name,\n                    size,\n                    hash,\n                }\n            },\n            bundle {\n                id,\n                name,\n                size,\n                hash,\n            }\n        }\n    }\n}\n\nmutation SubmitTaskResult($input: SubmitTaskResultInput!) {\n    submitTaskResult(input: $input) {\n        id\n    }\n}" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    #[derive(Clone)]
    pub enum SessionHostPlatform {
        Windows,
        Linux,
        MacOS,
        BSD,
        Unknown,
        Other(String),
    }
    impl ::serde::Serialize for SessionHostPlatform {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                SessionHostPlatform::Windows => "Windows",
                SessionHostPlatform::Linux => "Linux",
                SessionHostPlatform::MacOS => "MacOS",
                SessionHostPlatform::BSD => "BSD",
                SessionHostPlatform::Unknown => "Unknown",
                SessionHostPlatform::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for SessionHostPlatform {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "Windows" => Ok(SessionHostPlatform::Windows),
                "Linux" => Ok(SessionHostPlatform::Linux),
                "MacOS" => Ok(SessionHostPlatform::MacOS),
                "BSD" => Ok(SessionHostPlatform::BSD),
                "Unknown" => Ok(SessionHostPlatform::Unknown),
                _ => Ok(SessionHostPlatform::Other(s)),
            }
        }
    }
    #[derive(Serialize, Clone)]
    pub struct ClaimTasksInput {
        pub principal: String,
        pub hostname: String,
        #[serde(rename = "hostPlatform")]
        pub host_platform: SessionHostPlatform,
        #[serde(rename = "hostPrimaryIP")]
        pub host_primary_ip: Option<String>,
        #[serde(rename = "sessionIdentifier")]
        pub session_identifier: String,
        #[serde(rename = "hostIdentifier")]
        pub host_identifier: String,
        #[serde(rename = "agentIdentifier")]
        pub agent_identifier: String,
    }
    #[derive(Serialize, Clone)]
    pub struct Variables {
        pub input: ClaimTasksInput,
    }
    impl Variables {}
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ResponseData {
        #[serde(rename = "claimTasks")]
        pub claim_tasks: Vec<ClaimTasksClaimTasks>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasks {
        pub id: ID,
        pub job: ClaimTasksClaimTasksJob,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksJob {
        pub id: ID,
        pub name: String,
        pub parameters: Option<String>,
        pub tome: ClaimTasksClaimTasksJobTome,
        pub bundle: Option<ClaimTasksClaimTasksJobBundle>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksJobTome {
        pub id: ID,
        pub name: String,
        pub description: String,
        #[serde(rename = "paramDefs")]
        pub param_defs: Option<String>,
        pub eldritch: String,
        pub files: Option<Vec<ClaimTasksClaimTasksJobTomeFiles>>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksJobTomeFiles {
        pub id: ID,
        pub name: String,
        pub size: Int,
        pub hash: String,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksJobBundle {
        pub id: ID,
        pub name: String,
        pub size: Int,
        pub hash: String,
    }
}
impl graphql_client::GraphQLQuery for ClaimTasks {
    type Variables = claim_tasks::Variables;
    type ResponseData = claim_tasks::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: claim_tasks::QUERY,
            operation_name: claim_tasks::OPERATION_NAME,
        }
    }
}
pub struct SubmitTaskResult;
pub mod submit_task_result {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "SubmitTaskResult";
    pub const QUERY : & str = "mutation ClaimTasks($input: ClaimTasksInput!) {\n    claimTasks(input: $input) {\n        id,\n        job {\n            id,\n            name,\n            parameters,\n            tome {\n                id,\n                name,\n                description,\n                paramDefs,\n                eldritch,\n                files {\n                    id,\n                    name,\n                    size,\n                    hash,\n                }\n            },\n            bundle {\n                id,\n                name,\n                size,\n                hash,\n            }\n        }\n    }\n}\n\nmutation SubmitTaskResult($input: SubmitTaskResultInput!) {\n    submitTaskResult(input: $input) {\n        id\n    }\n}" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    type Time = crate::scalars::Time;
    #[derive(Serialize, Clone)]
    pub struct SubmitTaskResultInput {
        #[serde(rename = "taskID")]
        pub task_id: ID,
        #[serde(rename = "execStartedAt")]
        pub exec_started_at: Time,
        #[serde(rename = "execFinishedAt")]
        pub exec_finished_at: Option<Time>,
        pub output: String,
        pub error: Option<String>,
    }
    #[derive(Serialize, Clone)]
    pub struct Variables {
        pub input: SubmitTaskResultInput,
    }
    impl Variables {}
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ResponseData {
        #[serde(rename = "submitTaskResult")]
        pub submit_task_result: Option<SubmitTaskResultSubmitTaskResult>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct SubmitTaskResultSubmitTaskResult {
        pub id: ID,
    }
}
impl graphql_client::GraphQLQuery for SubmitTaskResult {
    type Variables = submit_task_result::Variables;
    type ResponseData = submit_task_result::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: submit_task_result::QUERY,
            operation_name: submit_task_result::OPERATION_NAME,
        }
    }
}
