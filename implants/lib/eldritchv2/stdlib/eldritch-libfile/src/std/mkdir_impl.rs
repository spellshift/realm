use alloc::format;
use alloc::string::String;
use ::std::fs;

pub fn mkdir(path: String, parent: Option<bool>) -> Result<(), String> {
    if parent.unwrap_or(false) {
        fs::create_dir_all(&path)
    } else {
        fs::create_dir(&path)
    }
    .map_err(|e| format!("Failed to create directory {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_mkdir_parent() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let base_path = tmp_dir.path();

        let sub_dir = base_path.join("sub/deep");
        let sub_dir_str = sub_dir.to_string_lossy().to_string();

        // Without parent=true, should fail
        assert!(mkdir(sub_dir_str.clone(), Some(false)).is_err());

        // With parent=true, should succeed
        mkdir(sub_dir_str.clone(), Some(true)).unwrap();
        assert!(sub_dir.exists());
        assert!(sub_dir.is_dir());
    }
}
