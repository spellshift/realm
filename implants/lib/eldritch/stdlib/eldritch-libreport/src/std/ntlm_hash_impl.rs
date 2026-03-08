use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_credential_request;
use pb::{c2, eldritch};

pub fn ntlm_hash(
    agent: Arc<dyn Agent>,
    context: Context,
    username: String,
    hash: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: hash,
        kind: 3, // KIND_NTLM_HASH
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
