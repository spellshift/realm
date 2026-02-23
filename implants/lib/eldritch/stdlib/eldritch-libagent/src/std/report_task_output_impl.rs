use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::{
    ReportShellTaskOutputMessage, ReportTaskOutputMessage, ShellTaskOutput, TaskError, TaskOutput,
    report_output_request,
};

#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_task_output(
    agent: Arc<dyn Agent>,
    context: Context,
    output: String,
    error: Option<String>,
) -> Result<(), String> {
    let task_error = error.map(|msg| TaskError { msg });

    let message_val = match context {
        Context::Task(tc) => {
            let output_msg = TaskOutput {
                id: tc.task_id,
                output,
                error: task_error,
                exec_started_at: None,
                exec_finished_at: None,
            };
            Some(report_output_request::Message::TaskOutput(
                ReportTaskOutputMessage {
                    context: Some(tc),
                    output: Some(output_msg),
                },
            ))
        }
        Context::ShellTask(stc) => {
            let output_msg = ShellTaskOutput {
                id: stc.shell_task_id,
                output,
                error: task_error,
                exec_started_at: None,
                exec_finished_at: None,
            };
            Some(report_output_request::Message::ShellTaskOutput(
                ReportShellTaskOutputMessage {
                    context: Some(stc),
                    output: Some(output_msg),
                },
            ))
        }
    };

    let req = c2::ReportOutputRequest {
        message: message_val,
    };
    agent.report_output(req).map(|_| ())
}
