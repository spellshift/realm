use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use ::std::fs;

pub fn read_binary(path: String) -> Result<Vec<u8>, String> {
    fs::read(&path).map_err(|e| format!("Failed to read file {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_binary() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        fs::write(&path, &data).unwrap();

        let read_data = read_binary(path).unwrap();
        assert_eq!(read_data, data);
    }
}
