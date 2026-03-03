use alloc::format;
use eldritch_core::Bytes;

pub fn md5(data: Bytes) -> Result<String, String> {
    Ok(format!("{:02x}", md5::compute(data.as_ref())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let data = Bytes::from(b"hello world");
        let hash = md5(data).unwrap();
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }
}
