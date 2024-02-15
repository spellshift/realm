mod fetch_asset;
mod report_credential;
mod report_error;
mod report_file;
mod report_finish;
mod report_process_list;
mod report_start;
mod report_text;

pub use fetch_asset::FetchAssetMessage;
pub use report_credential::ReportCredentialMessage;
pub use report_error::ReportErrorMessage;
pub use report_file::ReportFileMessage;
pub use report_finish::ReportFinishMessage;
pub use report_process_list::ReportProcessListMessage;
pub use report_start::ReportStartMessage;
pub use report_text::ReportTextMessage;
pub use transport::Transport;

use anyhow::Result;
use derive_more::From;
use std::future::Future;

#[cfg(debug_assertions)]
use derive_more::Display;

pub trait Dispatcher {
    fn dispatch(self, transport: &mut impl Transport) -> impl Future<Output = Result<()>> + Send;
}

#[cfg_attr(debug_assertions, derive(Debug, Display))]
#[derive(From, Clone)]
pub enum Message {
    #[cfg_attr(debug_assertions, display(fmt = "FetchAsset"))]
    FetchAsset(FetchAssetMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportCredential"))]
    ReportCredential(ReportCredentialMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportError"))]
    ReportError(ReportErrorMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportFile"))]
    ReportFile(ReportFileMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportProcessList"))]
    ReportProcessList(ReportProcessListMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportText"))]
    ReportText(ReportTextMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportStart"))]
    ReportStart(ReportStartMessage),

    #[cfg_attr(debug_assertions, display(fmt = "ReportFinish"))]
    ReportFinish(ReportFinishMessage),
}

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

            Self::ReportStart(msg) => msg.dispatch(transport).await,
            Self::ReportFinish(msg) => msg.dispatch(transport).await,
        }
    }
}

pub(crate) fn aggregate(messages: Vec<Message>) -> Vec<Message> {
    messages
}
