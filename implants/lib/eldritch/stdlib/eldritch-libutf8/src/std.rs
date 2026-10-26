use super::Utf8Library;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod decode_impl;
pub mod encode_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Utf8Library)]
pub struct StdUtf8Library;

impl Utf8Library for StdUtf8Library {
    fn decode(&self, content: Vec<u8>) -> Result<String, String> {
        decode_impl::decode(content)
    }

    fn encode(&self, content: String) -> Result<Vec<u8>, String> {
        encode_impl::encode(content)
    }
}
