use alloc::format;
use alloc::string::String;
use ::std::fs;

pub fn read(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file {path}: {e}"))
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
