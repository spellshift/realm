use alloc::string::ToString;
use alloc::vec;
use bytes::Bytes;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;

pub fn bytes(len: i64) -> Result<Bytes, String> {
    if len < 0 {
        return Err("Length cannot be negative".to_string());
    }
    let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
    let mut buf = vec![0u8; len as usize];
    rng.fill(&mut buf[..]);
    Ok(Bytes::from(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        let b = bytes(10).unwrap();
        assert_eq!(b.len(), 10);
    }

    #[test]
    fn test_bytes_negative() {
        let b = bytes(-1);
        assert!(b.is_err());
        assert_eq!(b.err().unwrap(), "Length cannot be negative");
    }
}
