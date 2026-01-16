use super::{EventsLibrary, ON_CALLBACK_END, ON_CALLBACK_START, ON_TASK_END, ON_TASK_START};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use spin::RwLock;

// The global registry for event callbacks
pub static EVENT_REGISTRY: RwLock<BTreeMap<String, Vec<Value>>> = RwLock::new(BTreeMap::new());

#[eldritch_library_impl(EventsLibrary)]
#[derive(Debug, Default)]
pub struct StdEventsLibrary {}

impl StdEventsLibrary {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventsLibrary for StdEventsLibrary {
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec![
            ON_CALLBACK_START.to_string(),
            ON_CALLBACK_END.to_string(),
            ON_TASK_START.to_string(),
            ON_TASK_END.to_string(),
        ])
    }

    fn register(&self, event: Value, f: Value) -> Result<(), String> {
        let event_name = match event {
            Value::String(s) => s,
            _ => return Err("Event name must be a string".to_string()),
        };

        match f {
            Value::Function(_) | Value::NativeFunction(_, _) | Value::NativeFunctionWithKwargs(_, _) | Value::BoundMethod(_, _) => {
                let mut registry = EVENT_REGISTRY.write();
                registry.entry(event_name).or_default().push(f);
                Ok(())
            }
            _ => Err("Callback must be a function".to_string()),
        }
    }
}

/// Triggers an event and executes all registered callbacks.
pub fn trigger_event(interp: &mut eldritch_core::Interpreter, event: &str, args: Vec<Value>) {
    let callbacks = {
        let registry = EVENT_REGISTRY.read();
        registry.get(event).cloned()
    };

    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            if let Err(e) = interp.call_value(&callback, &args, &BTreeMap::new()) {
                let printer = interp.env.read().printer.clone();
                printer.print_err(&eldritch_core::Span::new(0, 0, 0), &alloc::format!("Event callback error for event '{}': {}", event, e));
            }
        }
    }
}
