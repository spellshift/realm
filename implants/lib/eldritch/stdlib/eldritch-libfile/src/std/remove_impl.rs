use ::std::fs;
use ::std::path::Path;
use alloc::format;
use alloc::string::String;
use glob::glob;

pub fn remove(path: String) -> Result<(), String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        for match_path in paths.flatten() {
            if match_path.is_dir() {
                fs::remove_dir_all(&match_path)
            } else {
                fs::remove_file(&match_path)
            }
            .map_err(|e| format!("Failed to remove {}: {e}", match_path.to_string_lossy()))?;
        }
        Ok(())
    } else {
        let p = Path::new(&path);
        if p.is_dir() {
            fs::remove_dir_all(p)
        } else {
            fs::remove_file(p)
        }
        .map_err(|e| format!("Failed to remove {path}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remove() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Note: NamedTempFile removes on drop.
        // But we can check removal.
        remove(path.clone()).unwrap();
        assert!(!Path::new(&path).exists());
    }
}
