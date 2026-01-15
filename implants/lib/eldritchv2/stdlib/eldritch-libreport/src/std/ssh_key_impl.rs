use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, TaskContext};
use pb::{c2, eldritch};

pub fn ssh_key(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    username: String,
    key: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: key,
        kind: 2, // KIND_SSH_KEY
    };
    let req = c2::ReportCredentialRequest {
        context: Some(task_context.into()),
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
