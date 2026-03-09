use ::std::fs;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use glob::glob;

pub fn read_binary(path: String) -> Result<Vec<u8>, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut result = Vec::new();
        let paths = glob(&path).map_err(|e| format!("Invalid glob pattern {path}: {e}"))?;
        let mut found = false;
        for entry in paths {
            if let Ok(match_path) = entry {
                if match_path.is_file() {
                    found = true;
                    let mut data = fs::read(&match_path).map_err(|e| {
                        format!("Failed to read file {}: {e}", match_path.to_string_lossy())
                    })?;
                    result.append(&mut data);
                }
            }
        }
        if !found {
            return Err(format!("No files found matching pattern {path}"));
        }
        Ok(result)
    } else {
        fs::read(&path).map_err(|e| format!("Failed to read file {path}: {e}"))
    }
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
