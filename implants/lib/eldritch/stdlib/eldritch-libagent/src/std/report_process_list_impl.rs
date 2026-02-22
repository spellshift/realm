use alloc::string::String;
use alloc::sync::Arc;

use crate::ProcessListWrapper;
use pb::c2::TaskContext;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_process_list(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    list: ProcessListWrapper,
) -> Result<(), String> {
    let req = c2::ReportProcessListRequest {
        context: Some(c2::report_process_list_request::Context::TaskContext(
            task_context,
        )),
        list: Some(list.0),
    };
    agent.report_process_list(req).map(|_| ())
}
