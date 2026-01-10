use alloc::string::String;
use alloc::sync::Arc;

use crate::FileWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn report_file(
    agent: Arc<dyn Agent>,
    task_id: i64,
    jwt: String,
    file: FileWrapper,
) -> Result<(), String> {
    let req = c2::ReportFileRequest {
        task_id,
        chunk: Some(file.0),
        jwt,
    };
    agent.report_file(req).map(|_| ())
}
