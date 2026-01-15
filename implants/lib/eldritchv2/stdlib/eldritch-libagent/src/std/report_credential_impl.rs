use alloc::string::String;
use alloc::sync::Arc;

use super::TaskContext;
use crate::CredentialWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_credential(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    credential: CredentialWrapper,
) -> Result<(), String> {
    let req = c2::ReportCredentialRequest {
        context: Some(task_context.into()),
        credential: Some(credential.0),
    };
    agent.report_credential(req).map(|_| ())
}
