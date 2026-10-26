use alloc::string::String;
use alloc::vec::Vec;

pub fn encode(content: String) -> Result<Vec<u8>, String> {
    Ok(content.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_encode_utf8_basic() {
        let res = encode("Hello".into()).unwrap();
        assert_eq!(res, vec![b'H', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_encode_utf8_empty() {
        let res = encode("".into()).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_encode_utf8_unicode() {
        let res = encode("🚀".into()).unwrap();
        assert_eq!(res, vec![0xf0, 0x9f, 0x9a, 0x80]);
    }
}
