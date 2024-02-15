mod fetch_asset;
mod report_credential;
mod report_error;
mod report_file;
mod report_process_list;
mod report_text;

pub use fetch_asset::FetchAsset;
pub use report_credential::ReportCredential;
pub use report_error::ReportError;
pub use report_file::ReportFile;
pub use report_process_list::ReportProcessList;
pub use report_text::ReportText;
pub use transport::Transport;

use anyhow::Result;
use derive_more::From;
use std::future::Future;

pub trait Dispatcher {
    fn dispatch(self, transport: &mut impl Transport) -> impl Future<Output = Result<()>> + Send;
}

#[derive(From, Clone)]
pub enum Message {
    FetchAsset(FetchAsset),
    ReportCredential(ReportCredential),
    ReportError(ReportError),
    ReportFile(ReportFile),
    ReportProcessList(ReportProcessList),
    ReportText(ReportText),
}

impl Dispatcher for Message {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        match self {
            Self::FetchAsset(msg) => msg.dispatch(transport).await,
            Self::ReportCredential(msg) => msg.dispatch(transport).await,
            Self::ReportError(msg) => msg.dispatch(transport).await,
            Self::ReportFile(msg) => msg.dispatch(transport).await,
            Self::ReportProcessList(msg) => msg.dispatch(transport).await,
            Self::ReportText(msg) => msg.dispatch(transport).await,
        }
    }
}

pub(crate) fn aggregate(messages: Vec<Message>) -> Vec<Message> {
    messages
}
