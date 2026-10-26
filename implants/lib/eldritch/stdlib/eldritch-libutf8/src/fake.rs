use super::Utf8Library;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Utf8Library)]
pub struct Utf8LibraryFake;

impl Utf8Library for Utf8LibraryFake {
    fn decode(&self, content: Vec<u8>) -> Result<String, String> {
        String::from_utf8(content).map_err(|e| format!("Invalid UTF-8: {e}"))
    }

    fn encode(&self, content: String) -> Result<Vec<u8>, String> {
        Ok(content.into_bytes())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_fake_utf8() {
        let lib = Utf8LibraryFake;
        let encoded = lib.encode("Hi".into()).unwrap();
        assert_eq!(encoded, vec![b'H', b'i']);
        let decoded = lib.decode(encoded).unwrap();
        assert_eq!(decoded, "Hi");
    }
}
