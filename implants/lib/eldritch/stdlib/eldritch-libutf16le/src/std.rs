use super::Utf16leLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod decode_impl;
pub mod encode_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Utf16leLibrary)]
pub struct StdUtf16leLibrary;

impl Utf16leLibrary for StdUtf16leLibrary {
    fn decode(&self, content: Vec<u8>) -> Result<String, String> {
        decode_impl::decode_utf16le(content)
    }

    fn encode(&self, content: String) -> Result<Vec<u8>, String> {
        encode_impl::encode_utf16le(content)
    }
}
