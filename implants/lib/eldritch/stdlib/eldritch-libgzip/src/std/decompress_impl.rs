use alloc::string::{String, ToString};
use alloc::vec::Vec;
use flate2::read::GzDecoder;
use std::io::Read;

pub fn decompress_bytes(data: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut decoder = GzDecoder::new(data.as_slice());
    let mut decoded_data = Vec::new();
    decoder
        .read_to_end(&mut decoded_data)
        .map_err(|e| e.to_string())?;
    Ok(decoded_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_invalid_bytes() {
        let bad = vec![0x01, 0x02, 0x03];
        assert!(decompress_bytes(bad).is_err());
    }
}
