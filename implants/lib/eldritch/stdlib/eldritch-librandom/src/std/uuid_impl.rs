use alloc::string::{String, ToString};

pub fn uuid() -> Result<String, String> {
    Ok(uuid::Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        let u = uuid().unwrap();
        assert!(uuid::Uuid::parse_str(&u).is_ok());
    }
}
