use super::Utf16leLibrary;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(Utf16leLibrary)]
pub struct Utf16leLibraryFake;

impl Utf16leLibrary for Utf16leLibraryFake {
    fn decode(&self, content: Vec<u8>) -> Result<String, String> {
        if !content.len().is_multiple_of(2) {
            return Err("Input bytes length must be a multiple of 2".into());
        }

        let utf16_units: Vec<u16> = content
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        String::from_utf16(&utf16_units).map_err(|e| format!("Invalid UTF-16: {}", e))
    }

    fn encode(&self, content: String) -> Result<Vec<u8>, String> {
        let mut encoded: Vec<u8> = Vec::with_capacity(content.len() * 2);
        for c in content.encode_utf16() {
            let bytes = c.to_le_bytes();
            encoded.push(bytes[0]);
            encoded.push(bytes[1]);
        }
        Ok(encoded)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_fake_utf16le() {
        let lib = Utf16leLibraryFake;
        let encoded = lib.encode("Hi".into()).unwrap();
        assert_eq!(encoded, vec![b'H', 0, b'i', 0]);
        let decoded = lib.decode(encoded).unwrap();
        assert_eq!(decoded, "Hi");
    }
}
