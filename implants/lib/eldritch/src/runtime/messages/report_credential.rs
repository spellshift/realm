use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::{c2::ReportCredentialRequest};

#[derive(Clone)]
pub struct ReportCredential {
    pub(crate) req: ReportCredentialRequest,
}

impl Dispatcher for ReportCredential {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport.report_credential(self.req).await?;
        Ok(())
    }
}
