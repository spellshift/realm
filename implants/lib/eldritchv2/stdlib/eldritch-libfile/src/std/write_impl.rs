use alloc::format;
use alloc::string::String;
use ::std::fs;

pub fn write(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("Failed to write to file {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        write(path.clone(), "hello".to_string()).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "hello");
    }
}
