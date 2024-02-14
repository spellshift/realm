mod download_file;
mod report_error;
mod report_file;
mod report_process_list;
mod report_text;
mod transport;

use anyhow::{Error, Result};

pub use download_file::DownloadFile;
pub use report_error::ReportError;
pub use report_file::ReportFile;
pub use report_process_list::ReportProcessList;
pub use report_text::ReportText;
pub use transport::Transport;

pub trait Dispatcher {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()>;
}

pub enum Message {
    ReportText(ReportText),
    ReportError(ReportError),
    ReportProcessList(ReportProcessList),
    ReportFile(ReportFile),
    DownloadFile(DownloadFile),
}

impl Dispatcher for Message {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        match self {
            Self::ReportText(msg) => msg.dispatch(transport).await,
            Self::ReportError(msg) => msg.dispatch(transport).await,
            Self::ReportProcessList(msg) => msg.dispatch(transport).await,
            Self::ReportFile(msg) => msg.dispatch(transport).await,
            Self::DownloadFile(msg) => msg.dispatch(transport).await,
        }
    }
}
