use ::std::path::Path;
use alloc::format;
use alloc::string::String;
use glob::glob;

pub fn is_dir(path: String) -> Result<bool, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        if let Some(Ok(first_match)) = paths.next() {
            Ok(first_match.is_dir())
        } else {
            Ok(false)
        }
    } else {
        Ok(Path::new(&path).is_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_dir() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        assert!(!is_dir(path).unwrap());
        assert!(is_dir(dir_path).unwrap());
    }
}
