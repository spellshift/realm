use ::std::fs;
use ::std::io::Write;
use alloc::format;
use alloc::string::String;

pub fn append(path: String, content: String) -> Result<(), String> {
    let resolved_paths = crate::std::glob_util::resolve_paths(&path)?;
    for p in resolved_paths {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
            .map_err(|e| format!("Failed to open {}: {e}", p.display()))?;

        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to {}: {e}", p.display()))?;
    }

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
