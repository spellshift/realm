use ::std::fs;
use alloc::format;
use alloc::string::String;
use eldritch_core::Value;

pub fn write_binary(path: String, content: Value) -> Result<(), String> {
    let bytes = match content {
        Value::Bytes(b) => b,
        _ => return Err("content must be of type Bytes".into()),
    };
    fs::write(&path, bytes).map_err(|e| format!("Failed to write to file {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_binary() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        write_binary(path.clone(), Value::Bytes(data.clone())).unwrap();

        assert_eq!(fs::read(path).unwrap(), data);
    }
}
