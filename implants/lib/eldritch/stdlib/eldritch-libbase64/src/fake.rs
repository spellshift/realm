use super::Base64Library;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Base64Library)]
pub struct Base64LibraryFake;

impl Base64Library for Base64LibraryFake {
    fn decode(&self, content: String, _encode_type: Option<String>) -> Result<Vec<u8>, String> {
        if let Some(stripped) = content.strip_prefix("B64:") {
            Ok(stripped.as_bytes().to_vec())
        } else {
            Ok(format!("DECODED:{}", content).into_bytes())
        }
    }

    fn encode(&self, content: Vec<u8>, _encode_type: Option<String>) -> Result<String, String> {
        Ok(format!("B64:{}", String::from_utf8_lossy(&content)))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_base64_fake() {
        let b64 = Base64LibraryFake;
        let orig = b"hello world".to_vec();
        let b64_enc = b64.encode(orig.clone(), None).unwrap();
        let b64_dec = b64.decode(b64_enc, None).unwrap();
        assert_eq!(orig, b64_dec);
    }
}
