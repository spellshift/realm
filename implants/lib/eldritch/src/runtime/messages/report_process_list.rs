use super::{Dispatcher, Transport};
use anyhow::Result;
use api::pb::{c2::ReportProcessListRequest, eldritch::ProcessList};

pub struct ReportProcessList {
    id: i64,
    list: ProcessList,
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
