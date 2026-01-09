use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_task_output(
    agent: Arc<dyn Agent>,
    task_id: i64,
    output: String,
    error: Option<String>,
) -> Result<(), String> {
    let task_error = error.map(|msg| c2::TaskError { msg });
    let output_msg = c2::TaskOutput {
        id: task_id,
        output,
        error: task_error,
        exec_started_at: None,
        exec_finished_at: None,
    };
    let req = c2::ReportTaskOutputRequest {
        output: Some(output_msg),
    };
    agent.report_task_output(req).map(|_| ())
}
