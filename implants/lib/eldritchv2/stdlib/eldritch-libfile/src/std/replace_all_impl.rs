#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;

#[cfg(feature = "stdlib")]
pub fn replace_all(path: String, pattern: String, value: String) -> Result<(), String> {
    replace_impl(path, pattern, value).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn replace_all(
    _path: alloc::string::String,
    _pattern: alloc::string::String,
    _value: alloc::string::String,
) -> Result<(), alloc::string::String> {
    Err("replace_all requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn replace_impl(path: String, pattern: String, value: String) -> AnyhowResult<()> {
    use regex::bytes::{NoExpand, Regex};
    use std::fs;

    let data = fs::read(&path)?;
    let re = Regex::new(&pattern)?;

    let result = re.replace_all(&data, NoExpand(value.as_bytes()));

    fs::write(&path, result)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_replace_all() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        fs::write(&path, "hello world hello universe").unwrap();

        replace_all(path.clone(), "hello".to_string(), "hi".to_string()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hi world hi universe");
    }
}
