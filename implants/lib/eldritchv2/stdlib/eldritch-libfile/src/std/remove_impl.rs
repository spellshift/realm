use ::std::fs;
use ::std::path::Path;
use alloc::format;
use alloc::string::String;

pub fn remove(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.is_dir() {
        fs::remove_dir_all(p)
    } else {
        fs::remove_file(p)
    }
    .map_err(|e| format!("Failed to remove {path}: {e}"))
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
