use ::std::fs;
use alloc::format;
use alloc::string::String;

pub fn read(path: String) -> Result<String, String> {
    let resolved = crate::std::glob_util::resolve_first_path(&path)?;
    fs::read_to_string(&resolved).map_err(|e| format!("Failed to read file {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        fs::write(&path, "hello").unwrap();

        assert_eq!(read(path).unwrap(), "hello");
    }
}
