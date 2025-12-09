use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{c2::ReportProcessListRequest, config::Config, eldritch::ProcessList};

/*
 * ReportProcessListMessage reports a process list snapshot captured by this tome's evaluation.
 * It should never be send with a partial listing, only with full process lists.
 */
#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportProcessListMessage {
    pub(crate) id: i64,
    pub(crate) list: ProcessList,
}

impl AsyncDispatcher for ReportProcessListMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_process_list(ReportProcessListRequest {
                task_id: self.id,
                list: Some(self.list),
            })
            .await?;
        Ok(())
    }
}
