use ::std::path::Path;
use alloc::format;
use alloc::string::String;
use glob::glob;

pub fn is_file(path: String) -> Result<bool, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        let first_match = paths.next();
        let second_match = paths.next();

        if second_match.is_some() {
            return Err(format!(
                "Globbing not supported for multiple paths (pattern: {path})"
            ));
        }

        if let Some(Ok(match_path)) = first_match {
            Ok(match_path.is_file())
        } else {
            Ok(false)
        }
    } else {
        Ok(Path::new(&path).is_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        assert!(is_file(path).unwrap());
        assert!(!is_file(dir_path).unwrap());
    }
}
