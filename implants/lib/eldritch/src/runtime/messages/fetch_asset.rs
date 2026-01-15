use super::AsyncDispatcher;
use anyhow::Result;
use pb::c2::{FetchAssetRequest, FetchAssetResponse, TaskContext};
pub use pb::config::Config;
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

impl AsyncDispatcher for FetchAssetMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .fetch_asset(
                FetchAssetRequest {
                    name: self.name,
                    context: Some(TaskContext {
                        task_id: 0,
                        jwt: "no_jwt".to_string(),
                    }),
                },
                self.tx,
            )
            .await?;
        Ok(())
    }
}

#[cfg(debug_assertions)]
impl PartialEq for FetchAssetMessage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
