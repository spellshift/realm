use alloc::string::String;
use alloc::sync::Arc;

use crate::ProcessListWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_process_list(
    agent: Arc<dyn Agent>,
    task_id: i64,
    jwt: String,
    list: ProcessListWrapper,
) -> Result<(), String> {
    let req = c2::ReportProcessListRequest {
        task_id,
        list: Some(list.0),
        jwt,
    };
    agent.report_process_list(req).map(|_| ())
}
