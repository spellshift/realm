use super::ReportLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod file_impl;
pub mod process_list_impl;
pub mod ssh_key_impl;
pub mod user_password_impl;

#[eldritch_library_impl(ReportLibrary)]
pub struct StdReportLibrary {
    pub agent: Arc<dyn Agent>,
    pub task_id: i64,
    pub jwt: String,
}

impl core::fmt::Debug for StdReportLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdReportLibrary")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl StdReportLibrary {
    pub fn new(agent: Arc<dyn Agent>, task_id: i64, jwt: String) -> Self {
        Self {
            agent,
            task_id,
            jwt,
        }
    }
}

impl ReportLibrary for StdReportLibrary {
    fn file(&self, path: String) -> Result<(), String> {
        file_impl::file(self.agent.clone(), self.task_id, self.jwt.clone(), path)
    }

    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String> {
        process_list_impl::process_list(self.agent.clone(), self.task_id, self.jwt.clone(), list)
    }

    fn ssh_key(&self, username: String, key: String) -> Result<(), String> {
        ssh_key_impl::ssh_key(
            self.agent.clone(),
            self.task_id,
            self.jwt.clone(),
            username,
            key,
        )
    }

    fn user_password(&self, username: String, password: String) -> Result<(), String> {
        user_password_impl::user_password(
            self.agent.clone(),
            self.task_id,
            self.jwt.clone(),
            username,
            password,
        )
    }
}
