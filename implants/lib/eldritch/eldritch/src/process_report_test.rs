#[cfg(feature = "stdlib")]
use crate::Interpreter;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::sync::Arc;
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use eldritch_agent::Context;
#[cfg(feature = "stdlib")]
use eldritch_mockagent::MockAgent;
#[cfg(feature = "stdlib")]
use pb::c2::TaskContext;

#[test]
#[cfg(feature = "stdlib")]
fn test_report_process_list_integration() {
    {
        use eldritch_libassets::std::EmptyAssets;

        let agent = Arc::new(MockAgent::new());
        let task_context = TaskContext {
            task_id: 123,
            jwt: "test_jwt".to_string(),
        };
        let context = Context::Task(task_context);
        let backend = Arc::new(EmptyAssets {});

        let mut interp = Interpreter::new().with_default_libs().with_context(
            agent,
            context,
            Vec::new(),
            backend,
        );

        let code = "report.process_list(process.list())";
        let result = interp.interpret(code);

        assert!(result.is_ok(), "Interpretation failed: {:?}", result.err());
    }
}
