use alloc::format;
use bytes::Bytes;
use sha1::Sha1;
use sha2::Digest;

pub fn sha1(data: Bytes) -> Result<String, String> {
    let mut hasher = Sha1::new();
    hasher.update(data.as_ref());
    Ok(format!("{:02x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1() {
        let data = Bytes::from_static(b"hello world");
        let hash = sha1(data).unwrap();
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
