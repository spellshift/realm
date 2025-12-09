use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskOutput},
    config::Config,
};

/*
 * ReportTextMessage reports textual output (e.g. from `print()`) created by this tome's evaluation.
 */
#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportTextMessage {
    pub(crate) id: i64,
    pub(crate) text: String,
}

impl ReportTextMessage {
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

impl AsyncDispatcher for ReportTextMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: self.text,
                    exec_started_at: None,
                    exec_finished_at: None,
                    error: None,
                }),
            })
            .await?;
        Ok(())
    }
}
