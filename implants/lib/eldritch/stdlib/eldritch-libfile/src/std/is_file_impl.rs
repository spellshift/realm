use alloc::string::String;

pub fn is_file(path: String) -> Result<bool, String> {
    let resolved = match crate::std::glob_util::resolve_first_path(&path) {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };
    Ok(resolved.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        assert!(is_file(path).unwrap());
        assert!(!is_file(dir_path).unwrap());
    }
}
