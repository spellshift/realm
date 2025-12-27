#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::string::String;

#[cfg(feature = "stdlib")]
pub fn replace(path: String, pattern: String, value: String) -> Result<(), String> {
    replace_impl(path, pattern, value).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn replace(_path: alloc::string::String, _pattern: alloc::string::String, _value: alloc::string::String) -> Result<(), alloc::string::String> {
    Err("replace requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn replace_impl(path: String, pattern: String, value: String) -> AnyhowResult<()> {
    use std::fs;
    use regex::bytes::{NoExpand, Regex};

    let data = fs::read(&path)?;
    let re = Regex::new(&pattern)?;

    let result = re.replace(&data, NoExpand(value.as_bytes()));

    fs::write(&path, result)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_replace() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        fs::write(&path, "hello world hello universe").unwrap();

        replace(path.clone(), "hello".to_string(), "hi".to_string())
            .unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hi world hello universe");
    }
}
