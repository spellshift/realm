use anyhow::Result;

pub fn is_linux() -> Result<bool> {
    Ok(cfg!(target_os = "linux"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_linux() {
        let res = is_linux().unwrap();
        assert_eq!(res, cfg!(target_os = "linux"));
    }
}
