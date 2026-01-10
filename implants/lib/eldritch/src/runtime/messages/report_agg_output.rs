use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskError, TaskOutput},
    config::Config,
};
use prost_types::Timestamp;

/*
 * ReportAggOutput reports aggregated Text, Error, Start, and Finish messages
 * created by this tome's evaluation to help reduce load on the transpower.
 *
 * Prefer using Text, Error, Start, and Finish messages instead.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportAggOutputMessage {
    pub(crate) id: i64,
    pub(crate) error: Option<TaskError>,
    pub(crate) text: String,
    pub(crate) exec_started_at: Option<Timestamp>,
    pub(crate) exec_finished_at: Option<Timestamp>,
}

impl ReportAggOutputMessage {
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

impl AsyncDispatcher for ReportAggOutputMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: self.text,
                    exec_started_at: self.exec_started_at,
                    exec_finished_at: self.exec_finished_at,
                    error: self.error,
                }),
                jwt: "no_jwt".to_string(),
            })
            .await?;
        Ok(())
    }
}
