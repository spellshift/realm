use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::{c2, eldritch};

pub fn user_password(
    agent: Arc<dyn Agent>,
    task_id: i64,
    jwt: String,
    username: String,
    password: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: password,
        kind: 1, // KIND_PASSWORD
    };
    let req = c2::ReportCredentialRequest {
        task_id,
        credential: Some(cred),
        jwt,
    };
    agent.report_credential(req).map(|_| ())
}
