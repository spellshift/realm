use super::Dispatcher;
use anyhow::Result;
use pb::c2::{FetchAssetRequest, FetchAssetResponse};
use std::sync::mpsc::Sender;
use transport::Transport;

/*
 * FetchAssetMessage indicates that the owner of the corresponding `eldritch::Runtime` should send
 * an asset with the requested name to the provided sender (it may be sent in chunks).
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct FetchAssetMessage {
    pub(crate) name: String,
    pub(crate) tx: Sender<FetchAssetResponse>,
}

impl Dispatcher for FetchAssetMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .fetch_asset(FetchAssetRequest { name: self.name }, self.tx)
            .await?;
        Ok(())
    }
}
