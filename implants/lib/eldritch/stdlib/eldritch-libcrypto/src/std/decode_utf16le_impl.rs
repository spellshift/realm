use alloc::string::String;
use alloc::vec::Vec;

pub fn decode_utf16le(content: Vec<u8>) -> Result<String, String> {
    if content.len() % 2 != 0 {
        return Err("Input bytes length must be a multiple of 2".into());
    }

    let utf16_units: Vec<u16> = content
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    String::from_utf16(&utf16_units).map_err(|e| format!("Invalid UTF-16: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf16le() {
        let input = vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
        let expected = String::from("Hello");
        assert_eq!(decode_utf16le(input).unwrap(), expected);
    }

    #[test]
    fn test_decode_utf16le_invalid_length() {
        let input = vec![0x48, 0x00, 0x65];
        let err = decode_utf16le(input).unwrap_err();
        assert_eq!(err, "Input bytes length must be a multiple of 2");
    }

    #[test]
    fn test_decode_utf16le_invalid_utf16() {
        // High surrogate not followed by low surrogate
        let input = vec![0x00, 0xD8, 0x48, 0x00];
        let err = decode_utf16le(input).unwrap_err();
        assert!(err.contains("Invalid UTF-16"));
    }
}
