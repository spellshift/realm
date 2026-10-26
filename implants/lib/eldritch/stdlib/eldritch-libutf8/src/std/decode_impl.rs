use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn decode(content: Vec<u8>) -> Result<String, String> {
    String::from_utf8(content).map_err(|e| format!("Error decoding utf8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_decode_utf8_basic() {
        let bytes = vec![b'H', b'e', b'l', b'l', b'o'];
        let res = decode(bytes).unwrap();
        assert_eq!(res, "Hello");
    }

    #[test]
    fn test_decode_utf8_empty() {
        let res = decode(vec![]).unwrap();
        assert_eq!(res, "");
    }

    #[test]
    fn test_decode_utf8_unicode() {
        let bytes = "🚀".as_bytes().to_vec();
        let res = decode(bytes).unwrap();
        assert_eq!(res, "🚀");
    }

    #[test]
    fn test_decode_utf8_invalid() {
        let res = decode(vec![0xff, 0xfe]);
        assert!(res.is_err());
    }
}
