use anyhow::Result;

pub fn is_macos() -> Result<bool> {
    Ok(cfg!(target_os = "macos"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_macos() {
        let res = is_macos().unwrap();
        assert_eq!(res, cfg!(target_os = "macos"));
    }
}
