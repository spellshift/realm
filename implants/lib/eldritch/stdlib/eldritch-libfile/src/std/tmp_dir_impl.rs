use alloc::string::String;
use alloc::string::ToString;

pub fn tmp_dir() -> Result<String, String> {
    let tmp_dir =
        tempfile::tempdir().map_err(|e| alloc::format!("Failed to create temp directory: {e}"))?;
    let path = tmp_dir.keep();
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert temp path to string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmp_dir() {
        let res = tmp_dir().unwrap();
        let path = std::path::Path::new(&res);
        assert!(path.is_absolute());
        assert!(path.is_dir());
        // Cleanup
        let _ = ::std::fs::remove_dir(&res);
    }
}
