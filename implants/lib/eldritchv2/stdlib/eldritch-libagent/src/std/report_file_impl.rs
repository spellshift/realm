use alloc::string::String;
use alloc::sync::Arc;

use super::TaskContext;
use crate::FileWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_file(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    file: FileWrapper,
) -> Result<(), String> {
    let req = c2::ReportFileRequest {
        context: Some(task_context.into()),
        chunk: Some(file.0),
    };
    agent.report_file(req).map(|_| ())
}
