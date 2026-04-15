use alloc::string::String;
use alloc::string::ToString;

pub fn tmp_dir(name: Option<String>) -> Result<String, String> {
    let temp_dir = ::std::env::temp_dir();
    let dir_name = name.unwrap_or_else(|| {
        use alloc::format;
        format!(
            "eldritch_{}",
            ::std::time::SystemTime::now()
                .duration_since(::std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    });
    let path = temp_dir.join(dir_name);
    ::std::fs::create_dir_all(&path)
        .map_err(|e| alloc::format!("Failed to create temp directory: {e}"))?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert temp path to string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmp_dir_no_name() {
        let res = tmp_dir(None).unwrap();
        let path = std::path::Path::new(&res);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        // Cleanup
        let _ = ::std::fs::remove_dir(&res);
    }

    #[test]
    fn test_tmp_dir_with_name() {
        let res = tmp_dir(Some("eldritch_test_dir".to_string())).unwrap();
        let path = std::path::Path::new(&res);
        assert!(res.ends_with("eldritch_test_dir"));
        assert!(path.is_dir());
        // Cleanup
        let _ = ::std::fs::remove_dir_all(&res);
    }
}
