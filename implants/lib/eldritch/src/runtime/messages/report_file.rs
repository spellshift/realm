use super::{Dispatcher, Transport};
use anyhow::Result;


#[derive(Clone)]
pub struct ReportFile {
    id: i64,
    path: String,
}

impl Dispatcher for ReportFile {
    async fn dispatch(self, _transport: &mut impl Transport) -> Result<()> {
        Ok(())
    }
}
