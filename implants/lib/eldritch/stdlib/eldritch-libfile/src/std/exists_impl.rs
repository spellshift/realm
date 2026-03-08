use alloc::string::String;

pub fn exists(path: String) -> Result<bool, String> {
    let resolved = match crate::std::glob_util::resolve_first_path(&path, false) {
        Ok(p) => p,
        Err(_) => return Ok(false), // if glob matches nothing, it doesn't exist
    };
    Ok(resolved.exists())
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
