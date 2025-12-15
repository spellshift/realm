use super::{AssetsLibrary, RustEmbed};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_macros::eldritch_library_impl;
use pb::c2::FetchAssetRequest;
use transport::ActiveTransport;
use transport::Transport;

#[eldritch_library_impl(AssetsLibrary)]
pub struct StdAssetsLibrary<A: RustEmbed + Send + Sync> {
    phantom: core::marker::PhantomData<A>,
    agent: Arc<dyn Agent>,
    transport: ActiveTransport,
    task_id: i64,
}

impl<A: RustEmbed + Send + Sync> core::fmt::Debug for StdAssetsLibrary<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAssetsLibrary").finish()
    }
}

impl<A: RustEmbed + Send + Sync> StdAssetsLibrary<A> {
    pub fn new(agent: Arc<dyn Agent>, transport: ActiveTransport, task_id: i64) -> Self {
        Self {
            phantom: core::marker::PhantomData,
            agent,
            transport,
            task_id,
        }
    }
}

impl<A: RustEmbed + Send + Sync> AssetsLibrary for StdAssetsLibrary<A> {
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String> {
        match A::get(&name) {
            Some(file) => Ok(file.data.into_owned()),
            None => Err(alloc::format!("Asset '{}' not found", name)),
        }
    }

    fn fetch(&self, name: String) -> Result<Vec<u8>, String> {
        let (tx_std, rx_std) = std::sync::mpsc::channel();

        let mut t = self.transport.clone();
        let name_clone = name.clone();

        let task_future = async move {
            let req = FetchAssetRequest { name: name_clone };
            if let Err(_e) = t.fetch_asset(req, tx_std.clone()).await {
                 // Ignore error sending on channel as it means receiver dropped or transport failed
            }
        };

        // We need to construct the future.
        let fut = alloc::boxed::Box::pin(task_future);

        if let Err(e) = self.agent.spawn_subtask(self.task_id, alloc::format!("fetch_{}", name), fut) {
             return Err(e);
        }

        let mut data = Vec::new();
        // The transport::fetch_asset uses the sender to send FetchAssetResponse which contains chunk.
        // We collect them all.
        for resp in rx_std {
             data.extend_from_slice(&resp.chunk);
        }

        Ok(data)
    }

    fn read(&self, name: String) -> Result<String, String> {
        let bytes = self.read_binary(name)?;
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        let bytes = self.read_binary(src)?;
        std::fs::write(dest, bytes).map_err(|e| e.to_string())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Ok(A::iter().map(|s| s.to_string()).collect())
    }
}
