use super::{Dispatcher, Transport};
use anyhow::Result;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ReportFileMessage {
    pub(crate) id: i64,
    pub(crate) path: String,
}

impl Dispatcher for ReportFileMessage {
    async fn dispatch(self, _transport: &mut impl Transport) -> Result<()> {
        Ok(())
    }
}
