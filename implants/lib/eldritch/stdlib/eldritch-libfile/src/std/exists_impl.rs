use ::std::path::Path;
use alloc::format;
use alloc::string::String;
use glob::glob;

pub fn exists(path: String) -> Result<bool, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        Ok(paths.next().is_some())
    } else {
        Ok(Path::new(&path).exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_exists() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        assert!(exists(path).unwrap());
        assert!(!exists("nonexistent_file_12345".to_string()).unwrap());
    }
}
