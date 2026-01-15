use crate::Interpreter;
#[cfg(feature = "stdlib")]
use crate::agent::fake::AgentFake;
use eldritch_core::Value;
use std::sync::Arc;

fn create_interp() -> Interpreter {
    #[cfg(feature = "stdlib")]
    {
        use eldritch_libassets::std::EmptyAssets;

        let agent_mock = Arc::new(AgentFake);
        let task_id = 123;
        let backend = Arc::new(EmptyAssets {});
        Interpreter::new().with_default_libs().with_task_context(
            agent_mock,
            task_id,
            vec![],
            backend,
        )
    }
    #[cfg(not(feature = "stdlib"))]
    {
        Interpreter::new().with_default_libs()
    }
}

#[test]
fn test_events_constants() {
    let mut interp = create_interp();
    
    let val = interp.interpret("events.ON_CALLBACK_START").unwrap();
    assert_eq!(val, Value::String("on_callback_start".to_string()));
    
    let val = interp.interpret("events.ON_CALLBACK_END").unwrap();
    assert_eq!(val, Value::String("on_callback_end".to_string()));
}

#[test]
fn test_events_register() {
    let mut interp = create_interp();
    
    let code = r#"
called = [False]
def my_hook():
    called[0] = True

events.register(events.ON_CALLBACK_START, my_hook)
"#;
    interp.interpret(code).unwrap();
    
    // Manually trigger the event
    use crate::events::std::trigger_event;
    trigger_event(&mut interp.inner, "on_callback_start", vec![]);
    
    let val = interp.interpret("called[0]").unwrap();
    assert_eq!(val, Value::Bool(true));
}
