use ::std::fs::OpenOptions;
use ::std::io::Write;
use alloc::format;
use alloc::string::String;
use eldritch_core::Value;

pub fn append_binary(path: String, content: Value) -> Result<(), String> {
    let bytes = match content {
        Value::Bytes(b) => b,
        _ => return Err("content must be of type Bytes".into()),
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open file {path}: {e}"))?;

    file.write_all(&bytes)
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

        let data = String::from(" world");
        append_binary(path.clone(), Value::Bytes(data.into_bytes())).unwrap();

        let content = ::std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
    }
}
