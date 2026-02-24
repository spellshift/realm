use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_credential_request;
use pb::{c2, eldritch};

pub fn user_password(
    agent: Arc<dyn Agent>,
    context: Context,
    username: String,
    password: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: password,
        kind: eldritch::credential::Kind::Password as i32,
    };

    let context_val = match context {
        Context::Task(tc) => Some(report_credential_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_credential_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::ReportCredentialRequest {
        context: context_val,
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
