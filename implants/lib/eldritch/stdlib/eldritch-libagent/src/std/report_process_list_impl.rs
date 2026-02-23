use alloc::string::String;
use alloc::sync::Arc;

use crate::ProcessListWrapper;
use eldritch_agent::Context;
use pb::c2::report_process_list_request;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_process_list(
    agent: Arc<dyn Agent>,
    context: Context,
    list: ProcessListWrapper,
) -> Result<(), String> {
    let context_val = match context {
        Context::Task(tc) => Some(report_process_list_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_process_list_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::ReportProcessListRequest {
        context: context_val,
        list: Some(list.0),
    };
    agent.report_process_list(req).map(|_| ())
}
