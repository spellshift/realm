#![allow(clippy::all, warnings)]
pub struct ClaimTasks;
pub mod claim_tasks {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "ClaimTasks";
    pub const QUERY : & str = "mutation ClaimTasks($input: ClaimTasksInput!) {\n    claimTasks(input: $input) {\n        id,\n        quest {\n            id,\n            name,\n            parameters,\n            tome {\n                id,\n                name,\n                description,\n                paramDefs,\n                eldritch,\n                files {\n                    id,\n                    name,\n                    size,\n                    hash,\n                }\n            },\n            bundle {\n                id,\n                name,\n                size,\n                hash,\n            }\n        }\n    }\n}\n\nmutation SubmitTaskResult($input: SubmitTaskResultInput!) {\n    submitTaskResult(input: $input) {\n        id\n    }\n}" ;
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
    pub enum HostPlatform {
        Windows,
        Linux,
        MacOS,
        BSD,
        Unknown,
        Other(String),
    }
    impl ::serde::Serialize for HostPlatform {
        fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
            ser.serialize_str(match *self {
                HostPlatform::Windows => "Windows",
                HostPlatform::Linux => "Linux",
                HostPlatform::MacOS => "MacOS",
                HostPlatform::BSD => "BSD",
                HostPlatform::Unknown => "Unknown",
                HostPlatform::Other(ref s) => &s,
            })
        }
    }
    impl<'de> ::serde::Deserialize<'de> for HostPlatform {
        fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s: String = ::serde::Deserialize::deserialize(deserializer)?;
            match s.as_str() {
                "Windows" => Ok(HostPlatform::Windows),
                "Linux" => Ok(HostPlatform::Linux),
                "MacOS" => Ok(HostPlatform::MacOS),
                "BSD" => Ok(HostPlatform::BSD),
                "Unknown" => Ok(HostPlatform::Unknown),
                _ => Ok(HostPlatform::Other(s)),
            }
        }
    }
    #[derive(Serialize, Clone)]
    pub struct ClaimTasksInput {
        pub principal: String,
        pub hostname: String,
        #[serde(rename = "hostPlatform")]
        pub host_platform: HostPlatform,
        #[serde(rename = "hostPrimaryIP")]
        pub host_primary_ip: Option<String>,
        #[serde(rename = "beaconIdentifier")]
        pub beacon_identifier: String,
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
        pub quest: ClaimTasksClaimTasksQuest,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksQuest {
        pub id: ID,
        pub name: String,
        pub parameters: Option<String>,
        pub tome: ClaimTasksClaimTasksQuestTome,
        pub bundle: Option<ClaimTasksClaimTasksQuestBundle>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksQuestTome {
        pub id: ID,
        pub name: String,
        pub description: String,
        #[serde(rename = "paramDefs")]
        pub param_defs: Option<String>,
        pub eldritch: String,
        pub files: Option<Vec<ClaimTasksClaimTasksQuestTomeFiles>>,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksQuestTomeFiles {
        pub id: ID,
        pub name: String,
        pub size: Int,
        pub hash: String,
    }
    #[derive(Deserialize, Serialize, Clone)]
    pub struct ClaimTasksClaimTasksQuestBundle {
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
    pub const QUERY : & str = "mutation ClaimTasks($input: ClaimTasksInput!) {\n    claimTasks(input: $input) {\n        id,\n        quest {\n            id,\n            name,\n            parameters,\n            tome {\n                id,\n                name,\n                description,\n                paramDefs,\n                eldritch,\n                files {\n                    id,\n                    name,\n                    size,\n                    hash,\n                }\n            },\n            bundle {\n                id,\n                name,\n                size,\n                hash,\n            }\n        }\n    }\n}\n\nmutation SubmitTaskResult($input: SubmitTaskResultInput!) {\n    submitTaskResult(input: $input) {\n        id\n    }\n}" ;
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
