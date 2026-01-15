use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, TaskContext};
use pb::{c2, eldritch};

pub fn user_password(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    username: String,
    password: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: password,
        kind: 1, // KIND_PASSWORD
    };
    let req = c2::ReportCredentialRequest {
        context: Some(task_context.into()),
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
