use super::JsonLibrary;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod decode_impl;
pub mod encode_impl;
pub mod is_valid_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(JsonLibrary)]
pub struct StdJsonLibrary;

impl JsonLibrary for StdJsonLibrary {
    fn decode(&self, content: String) -> Result<Value, String> {
        decode_impl::from_json(content)
    }

    fn encode(&self, content: Value) -> Result<String, String> {
        encode_impl::to_json(content)
    }

    fn is_valid(&self, content: String) -> Result<bool, String> {
        is_valid_impl::is_json(content)
    }
}
