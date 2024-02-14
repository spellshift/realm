use super::{Dispatcher, Transport};
use anyhow::Result;

pub struct DownloadFile {
    name: String,
}

impl Dispatcher for DownloadFile {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        println!("TODO");
        Ok(())
    }
}
