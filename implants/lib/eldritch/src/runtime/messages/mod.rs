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

pub use fetch_asset::FetchAssetMessage;
pub(super) use reduce::reduce;
pub use report_credential::ReportCredentialMessage;
pub use report_error::ReportErrorMessage;
pub use report_file::ReportFileMessage;
pub use report_finish::ReportFinishMessage;
pub use report_process_list::ReportProcessListMessage;
pub use report_start::ReportStartMessage;
pub use report_text::ReportTextMessage;
pub use reverse_shell_pty::ReverseShellPTYMessage;
pub use transport::Transport;

use anyhow::Result;
use derive_more::{Display, From};
use report_agg_output::ReportAggOutputMessage;
use std::future::Future;

// Dispatcher defines the shared "dispatch" method used by all `Message` variants to send their data using a transport.
pub trait Dispatcher {
    fn dispatch(self, transport: &mut impl Transport) -> impl Future<Output = Result<()>> + Send;
}

/*
 * A Message from an Eldritch tome evaluation `tokio::task` to the owner of the corresponding `eldritch::Runtime`.
 * This enables eldritch library functions to communicate with the caller API, enabling structured data reporting
 * as well as resource requests (e.g. fetching assets).
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Display, From, Clone)]
pub enum Message {
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

// The Dispatcher implementation for `Message` simply calls the `dispatch()` implementation on the underlying variant.
impl Dispatcher for Message {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        #[cfg(debug_assertions)]
        log::debug!("dispatching message {:?}", self);

        match self {
            Self::FetchAsset(msg) => msg.dispatch(transport).await,

            Self::ReportCredential(msg) => msg.dispatch(transport).await,
            Self::ReportError(msg) => msg.dispatch(transport).await,
            Self::ReportFile(msg) => msg.dispatch(transport).await,
            Self::ReportProcessList(msg) => msg.dispatch(transport).await,
            Self::ReportText(msg) => msg.dispatch(transport).await,
            Self::ReportAggOutput(msg) => msg.dispatch(transport).await,
            Self::ReverseShellPTY(msg) => msg.dispatch(transport).await,

            Self::ReportStart(msg) => msg.dispatch(transport).await,
            Self::ReportFinish(msg) => msg.dispatch(transport).await,
        }
    }
}
