use ::std::fs;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use glob::glob;

pub fn read_binary(path: String) -> Result<Vec<u8>, String> {
    let target_path = if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        if let Some(Ok(first_match)) = paths.next() {
            first_match.to_string_lossy().into_owned()
        } else {
            return Err(format!("No files found matching pattern {path}"));
        }
    } else {
        path.clone()
    };

    fs::read(&target_path).map_err(|e| format!("Failed to read file {target_path}: {e}"))
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
