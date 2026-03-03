use alloc::format;
use bytes::Bytes;
use sha2::{Digest, Sha256};

pub fn sha256(data: Bytes) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(data.as_ref());
    Ok(format!("{:02x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let data = Bytes::from_static(b"hello world");
        let hash = sha256(data).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
