use eldritch_core::Value;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

use alloc::collections::BTreeMap;

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[eldritch_library("agent")]
pub trait AgentLibrary {
    #[eldritch_method]
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn get_id(&self) -> Result<String, String>;

    #[eldritch_method]
    fn get_platform(&self) -> Result<String, String>;

    #[eldritch_method]
    fn kill(&self) -> Result<(), String>;

    #[eldritch_method]
    fn set_config(&self, config: BTreeMap<String, Value>) -> Result<(), String>;

    #[eldritch_method]
    fn sleep(&self, secs: i64) -> Result<(), String>;
}
