use alloc::string::String;

pub fn is_json(content: String) -> Result<bool, String> {
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_json_object() -> Result<(), String> {
        let res = is_json(r#"{"test": "test"}"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_is_json_list() -> Result<(), String> {
        let res = is_json(r#"[1, "foo", false, null]"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_is_json_invalid() -> Result<(), String> {
        let res = is_json(r#"{"test":"#.to_string())?;
        assert!(!res);
        Ok(())
    }
}
