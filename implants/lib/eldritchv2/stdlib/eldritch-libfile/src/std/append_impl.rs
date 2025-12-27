use alloc::format;
use alloc::string::String;
use ::std::fs::OpenOptions;
use ::std::io::Write;

pub fn append(path: String, content: String) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open file {path}: {e}"))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to file {path}: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_append() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Write initial
        ::std::fs::write(&path, "hello").unwrap();

        append(path.clone(), " world".to_string()).unwrap();

        let content = ::std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
    }
}
