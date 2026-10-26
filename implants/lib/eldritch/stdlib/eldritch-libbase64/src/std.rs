use super::Base64Library;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;

pub mod decode_impl;
pub mod encode_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Base64Library)]
pub struct StdBase64Library;

impl Base64Library for StdBase64Library {
    fn decode(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        decode_impl::decode_b64(content, encode_type)
    }

    fn encode(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        encode_impl::encode_b64(content, encode_type)
    }
}
