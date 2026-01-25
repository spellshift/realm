use anyhow::Result;

pub fn is_windows() -> Result<bool> {
    Ok(cfg!(target_os = "windows"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_windows() {
        let res = is_windows().unwrap();
        assert_eq!(res, cfg!(target_os = "windows"));
    }
}
