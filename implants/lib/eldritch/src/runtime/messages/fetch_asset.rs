use super::Dispatcher;
use anyhow::Result;
use pb::c2::{FetchAssetRequest, FetchAssetResponse};
use std::sync::mpsc::Sender;
use transport::Transport;

#[derive(Clone)]
pub struct FetchAsset {
    pub(crate) req: FetchAssetRequest,
    pub(crate) tx: Sender<FetchAssetResponse>,
}

impl Dispatcher for FetchAsset {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport.fetch_asset(self.req, self.tx).await?;
        Ok(())
    }
}
