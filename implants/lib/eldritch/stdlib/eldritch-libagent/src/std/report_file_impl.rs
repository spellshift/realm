use alloc::string::String;
use alloc::sync::Arc;

use crate::FileWrapper;
use pb::c2::TaskContext;

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
        context: Some(c2::report_file_request::Context::TaskContext(task_context)),
        chunk: Some(file.0),
        kind: c2::ReportFileKind::Ondisk as i32,
    };
    agent.report_file(req).map(|_| ())
}
