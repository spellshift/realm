use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;

use crate::FileWrapper;
use eldritch_agent::Context;
use pb::c2::report_file_request;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_file(
    agent: Arc<dyn Agent>,
    context: Context,
    file: FileWrapper,
) -> Result<(), String> {
    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::ReportFileRequest {
        context: context_val,
        chunk: Some(file.0),
        kind: c2::ReportFileKind::Ondisk as i32,
    };
    agent
        .report_file(Box::new(alloc::vec![req].into_iter()))
        .map(|_| ())
}
