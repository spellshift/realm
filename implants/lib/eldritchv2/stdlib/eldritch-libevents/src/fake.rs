use super::EventsLibrary;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[eldritch_library_impl(EventsLibrary)]
#[derive(Debug, Default)]
pub struct EventsLibraryFake;

impl EventsLibrary for EventsLibraryFake {
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec!["fake_event".to_string()])
    }

    fn register(&self, _event: Value, _f: Value) -> Result<(), String> {
        Ok(())
    }
}
