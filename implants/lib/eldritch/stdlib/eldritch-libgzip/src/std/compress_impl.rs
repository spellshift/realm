use alloc::string::{String, ToString};
use alloc::vec::Vec;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

pub fn compress_bytes(data: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&data).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::super::decompress_impl::decompress_bytes;
    use super::*;

    #[test]
    fn test_compress_decompress_bytes() {
        let data = b"Hello GZIP world!".to_vec();
        let compressed = compress_bytes(data.clone()).unwrap();
        assert!(!compressed.is_empty());
        assert_ne!(compressed, data);

        let decompressed = decompress_bytes(compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}
