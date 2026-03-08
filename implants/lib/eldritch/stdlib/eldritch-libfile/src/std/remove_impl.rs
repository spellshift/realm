use ::std::fs;
use alloc::format;
use alloc::string::String;

pub fn remove(path: String) -> Result<(), String> {
    let resolved_paths = crate::std::glob_util::resolve_paths(&path)?;
    for p in resolved_paths {
        if p.is_dir() {
            fs::remove_dir_all(&p)
        } else {
            fs::remove_file(&p)
        }
        .map_err(|e| format!("Failed to remove {}: {e}", p.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
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
