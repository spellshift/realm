use alloc::string::String;
use alloc::vec::Vec;

pub fn encode_utf16le(content: String) -> Result<Vec<u8>, String> {
    let mut encoded: Vec<u8> = Vec::with_capacity(content.len() * 2);
    for c in content.encode_utf16() {
        let bytes = c.to_le_bytes();
        encoded.push(bytes[0]);
        encoded.push(bytes[1]);
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_utf16le() {
        let input = String::from("Hello");
        // H: 0x0048 -> 0x48 0x00
        // e: 0x0065 -> 0x65 0x00
        // l: 0x006C -> 0x6C 0x00
        // l: 0x006C -> 0x6C 0x00
        // o: 0x006F -> 0x6F 0x00
        let expected = vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
        assert_eq!(encode_utf16le(input).unwrap(), expected);
    }
}
