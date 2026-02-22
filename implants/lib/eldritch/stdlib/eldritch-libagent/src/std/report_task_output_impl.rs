use alloc::string::String;
use alloc::sync::Arc;
use pb::c2::TaskContext;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_task_output(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    output: String,
    error: Option<String>,
) -> Result<(), String> {
    let task_error = error.map(|msg| c2::TaskError { msg });
    let output_msg = c2::TaskOutput {
        id: task_context.task_id,
        output,
        error: task_error,
        exec_started_at: None,
        exec_finished_at: None,
    };
    let req = c2::ReportOutputRequest {
        output: Some(c2::report_output_request::Output::TaskOutput(output_msg)),
        context: Some(c2::report_output_request::Context::TaskContext(task_context)),
    };
    agent.report_task_output(req).map(|_| ())
}
