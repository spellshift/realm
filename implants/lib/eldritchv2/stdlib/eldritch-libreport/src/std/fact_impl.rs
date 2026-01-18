use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::c2::{ReportFactRequest, TaskContext};
use pb::eldritch::Fact;

pub fn fact(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    name: String,
    value: String,
) -> Result<(), String> {
    let req = ReportFactRequest {
        context: Some(task_context),
        fact: Some(Fact { name, value }),
    };

    agent.report_fact(req).map(|_| ())
}
