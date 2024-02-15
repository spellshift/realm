use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::{c2::ReportProcessListRequest, eldritch::ProcessList};

#[derive(Clone)]
pub struct ReportProcessList {
    pub(crate) id: i64,
    pub(crate) list: ProcessList,
}

impl Dispatcher for ReportProcessList {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_process_list(ReportProcessListRequest {
                task_id: self.id,
                list: Some(self.list),
            })
            .await?;
        Ok(())
    }
}
