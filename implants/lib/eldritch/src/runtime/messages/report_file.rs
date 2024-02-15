use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::c2::ReportProcessListRequest;

#[derive(Clone)]
pub struct ReportFile {
    id: i64,
    path: String,
}

impl Dispatcher for ReportFile {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        Ok(())
    }
}
