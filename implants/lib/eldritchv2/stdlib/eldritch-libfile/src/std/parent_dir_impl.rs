use ::std::path::Path;
use alloc::string::String;
use alloc::string::ToString;

pub fn parent_dir(path: String) -> Result<String, String> {
    let path = Path::new(&path);
    let parent = path
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;

    parent
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert path to string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_dir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let base_path = tmp_dir.path();
        let file_path = base_path.join("test.txt");

        let parent = parent_dir(file_path.to_string_lossy().to_string()).unwrap();
        // parent_dir returns string of parent path
        // On temp dir, it might be complex, but let's check it ends with what we expect or is equal
        assert_eq!(parent, base_path.to_string_lossy().to_string());

        // Test root parent (might fail on some envs if we can't read root, but logic should hold)
        // If we pass "/", parent is None -> Error
        #[cfg(unix)]
        assert!(parent_dir("/".to_string()).is_err());
    }
}
