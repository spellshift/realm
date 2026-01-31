use alloc::format;
use alloc::vec::Vec;

pub fn md5(data: Vec<u8>) -> Result<String, String> {
    Ok(format!("{:02x}", md5::compute(data)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let data = b"hello world".to_vec();
        let hash = md5(data).unwrap();
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }
}
