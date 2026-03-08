use ::std::fs;
use alloc::format;
use alloc::string::String;

pub fn mkdir(path: String, parent: Option<bool>) -> Result<(), String> {
    let resolved_paths = crate::std::glob_util::resolve_paths(&path, false)
        .unwrap_or_else(|_| alloc::vec![std::path::PathBuf::from(&path)]);
    for p in resolved_paths {
        let res = if parent.unwrap_or(false) {
            fs::create_dir_all(&p)
        } else {
            fs::create_dir(&p)
        };
        res.map_err(|e| format!("Failed to create directory {}: {e}", p.display()))?;
    }
    Ok(())
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
