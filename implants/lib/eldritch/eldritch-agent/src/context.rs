use pb::c2::{ShellTaskContext, TaskContext};

#[derive(Clone, Debug)]
pub enum ReportContext {
    Task(TaskContext),
    Shell(ShellTaskContext),
}

pub trait ContextProvider: Send + Sync {
    fn get_context(&self) -> ReportContext;
}

#[derive(Clone, Debug)]
pub struct StaticContextProvider {
    context: TaskContext,
}

impl StaticContextProvider {
    pub fn new(context: TaskContext) -> Self {
        Self { context }
    }
}

impl ContextProvider for StaticContextProvider {
    fn get_context(&self) -> ReportContext {
        ReportContext::Task(self.context.clone())
    }
}
