use alloc::string::String;
use alloc::sync::Arc;

use crate::CredentialWrapper;
use eldritch_agent::Context;
use pb::c2::report_credential_request;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_credential(
    agent: Arc<dyn Agent>,
    context: Context,
    credential: CredentialWrapper,
) -> Result<(), String> {
    let context_val = match context {
        Context::Task(tc) => Some(report_credential_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_credential_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::ReportCredentialRequest {
        context: context_val,
        credential: Some(credential.0),
    };
    agent.report_credential(req).map(|_| ())
}
