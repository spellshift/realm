#[cfg(feature = "stdlib")]
use super::list_impl::create_dict_from_file;
#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use anyhow::{Context, Result as AnyhowResult};
#[cfg(feature = "stdlib")]
use eldritch_core::Value;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
pub fn stat(path: String) -> Result<BTreeMap<String, Value>, String> {
    stat_impl(path).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn stat(
    _path: alloc::string::String,
) -> Result<
    alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>,
    alloc::string::String,
> {
    Err("stat requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn stat_impl(path: String) -> AnyhowResult<BTreeMap<String, Value>> {
    let path_buf = Path::new(&path);
    if !path_buf.exists() {
        anyhow::bail!("No such file or directory: {path}");
    }
    create_dict_from_file(path_buf)
        .with_context(|| alloc::format!("Failed to stat path: {path}"))
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_stat_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let dict = stat(path).unwrap();

        assert!(dict.contains_key("file_name"));
        assert!(dict.contains_key("absolute_path"));
        assert!(dict.contains_key("size"));
        assert!(dict.contains_key("owner"));
        assert!(dict.contains_key("group"));
        assert!(dict.contains_key("permissions"));
        assert!(dict.contains_key("times"));
        if let Some(Value::String(t)) = dict.get("type") {
            assert_eq!(t, "file");
        } else {
            panic!("type is not a string");
        }
    }

    #[test]
    fn test_stat_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_string_lossy().to_string();
        let dict = stat(path).unwrap();

        if let Some(Value::String(t)) = dict.get("type") {
            assert_eq!(t, "dir");
        } else {
            panic!("type is not a string");
        }
    }

    #[test]
    fn test_stat_nonexistent() {
        let result = stat("/this/path/does/not/exist/hopefully".to_string());
        assert!(result.is_err());
    }
}
