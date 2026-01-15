extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;

pub const ON_CALLBACK_START: &str = "on_callback_start";
pub const ON_CALLBACK_END: &str = "on_callback_end";
pub const ON_TASK_START: &str = "on_task_start";
pub const ON_TASK_END: &str = "on_task_end";

#[eldritch_library("events")]
/// The `events` library provides a mechanism for registering callbacks that are executed when specific agent events occur.
///
/// This allows you to:
/// - Hook into the agent's lifecycle (e.g., before or after a callback).
/// - Monitor task execution.
/// - Implement custom logic in response to agent activities.
pub trait EventsLibrary {
    #[eldritch_method]
    /// Returns a list of all available events.
    ///
    /// **Returns**
    /// - `List<str>`: A list of event names that can be registered.
    fn list(&self) -> Result<Vec<String>, String>;

    #[eldritch_method]
    /// Registers a callback function for a specific event.
    ///
    /// **Parameters**
    /// - `event` (`str`): The name of the event (e.g., `events.ON_CALLBACK_START`).
    /// - `f` (`function`): The callback function to execute.
    ///
    /// **Returns**
    /// - `None`
    fn register(&self, event: Value, f: Value) -> Result<(), String>;

    fn _eldritch_get_attr(&self, name: &str) -> Option<Value> {
        match name {
            "ON_CALLBACK_START" => Some(Value::String(ON_CALLBACK_START.to_string())),
            "ON_CALLBACK_END" => Some(Value::String(ON_CALLBACK_END.to_string())),
            "ON_TASK_START" => Some(Value::String(ON_TASK_START.to_string())),
            "ON_TASK_END" => Some(Value::String(ON_TASK_END.to_string())),
            _ => None,
        }
    }
}
