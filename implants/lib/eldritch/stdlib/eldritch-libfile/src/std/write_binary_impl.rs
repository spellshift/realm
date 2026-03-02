use ::std::fs;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn write_binary(path: String, content: Vec<u8>) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("Failed to write to file {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_binary() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let binary_content = vec![0xDE, 0xAD, 0xBE, 0xEF];
        write_binary(path.clone(), binary_content.clone()).unwrap();

        assert_eq!(fs::read(path).unwrap(), binary_content);
    }
}
