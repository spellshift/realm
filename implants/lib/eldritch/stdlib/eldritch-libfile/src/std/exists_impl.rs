use ::std::path::Path;
use alloc::string::String;

pub fn exists(path: String) -> Result<bool, String> {
    Ok(Path::new(&path).exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_exists() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        assert!(exists(path).unwrap());
        assert!(!exists("nonexistent_file_12345".to_string()).unwrap());
    }
}
