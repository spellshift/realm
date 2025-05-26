use anyhow::Result;

pub fn is_json(json: String) -> Result<bool> {
    match serde_json::from_str::<serde_json::Value>(&json) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_from_json_object() -> anyhow::Result<()> {
        let res = super::is_json(r#"{"test": "test"}"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_from_json_list() -> anyhow::Result<()> {
        let res = super::is_json(r#"[1, "foo", false, null]"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_from_json_invalid() -> anyhow::Result<()> {
        let res = super::is_json(r#"{"test":"#.to_string())?;
        assert!(!res);
        Ok(())
    }
}
