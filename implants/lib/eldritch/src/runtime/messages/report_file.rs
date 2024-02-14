use super::{Dispatcher, Transport};
use anyhow::Result;
use api::pb::c2::ReportProcessListRequest;

pub struct ReportFile {
    id: i64,
    path: String,
}

impl Dispatcher for ReportFile {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        Ok(())
    }
}
