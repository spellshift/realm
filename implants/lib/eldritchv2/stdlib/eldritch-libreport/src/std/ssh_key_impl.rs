use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::{c2, eldritch};

pub fn ssh_key(
    agent: Arc<dyn Agent>,
    task_id: i64,
    jwt: String,
    username: String,
    key: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: key,
        kind: 2, // KIND_SSH_KEY
    };
    let req = c2::ReportCredentialRequest {
        task_id,
        credential: Some(cred),
        jwt,
    };
    agent.report_credential(req).map(|_| ())
}
