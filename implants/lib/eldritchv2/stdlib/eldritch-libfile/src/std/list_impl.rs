#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;
#[cfg(feature = "stdlib")]
use std::fs;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
pub fn list(path: Option<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
    let path = path.unwrap_or_else(|| {
        ::std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "C:\\".to_string()
                } else {
                    "/".to_string()
                }
            })
    });
    list_impl(path).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn list(
    _path: Option<alloc::string::String>,
) -> Result<
    alloc::vec::Vec<alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>>,
    alloc::string::String,
> {
    Err("list requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn list_impl(path: String) -> AnyhowResult<Vec<BTreeMap<String, Value>>> {
    use glob::glob;

    let mut final_res = Vec::new();

    // Glob
    for entry in glob(&path)? {
        match entry {
            Ok(path_buf) => {
                if path_buf.is_dir() {
                    for entry in fs::read_dir(&path_buf)? {
                        let entry = entry?;
                        final_res.push(create_dict_from_file(&entry.path())?);
                    }
                } else {
                    final_res.push(create_dict_from_file(&path_buf)?);
                }
            }
            Err(e) => eprintln!("Glob error: {e:?}"),
        }
    }
    Ok(final_res)
}

#[cfg(feature = "stdlib")]
fn create_dict_from_file(path: &Path) -> AnyhowResult<BTreeMap<String, Value>> {
    use super::metadata_impl::get_file_info;

    let info = get_file_info(path)?;
    let mut dict = BTreeMap::new();

    dict.insert("file_name".to_string(), Value::String(info.file_name));
    dict.insert("type".to_string(), Value::String(info.file_type));
    dict.insert("size".to_string(), Value::Int(info.size as i64));
    dict.insert("permissions".to_string(), Value::String(info.permissions));
    dict.insert("owner".to_string(), Value::String(info.owner));
    dict.insert("group".to_string(), Value::String(info.group));
    dict.insert(
        "absolute_path".to_string(),
        Value::String(info.absolute_path),
    );
    dict.insert("modified".to_string(), Value::String(info.modified));

    Ok(dict)
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use regex::bytes::Regex;
    use tempfile::NamedTempFile;

    #[test]
    fn test_list_owner_group() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let files = list(Some(path)).unwrap();
        assert_eq!(files.len(), 1);
        let f = &files[0];

        assert!(f.contains_key("owner"));
        assert!(f.contains_key("group"));
        assert!(f.contains_key("absolute_path"));
        assert!(f.contains_key("modified"));

        // Check absolute_path
        if let Value::String(abs) = &f["absolute_path"] {
            assert!(!abs.is_empty());
            assert!(std::path::Path::new(abs).is_absolute());
        } else {
            panic!("absolute_path is not a string");
        }

        // Check modified time format
        if let Value::String(mod_time) = &f["modified"] {
            // Check format YYYY-MM-DD HH:MM:SS UTC
            let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} UTC$").unwrap();
            assert!(
                re.is_match(mod_time.as_bytes()),
                "Timestamp format mismatch: {}",
                mod_time
            );
        } else {
            panic!("modified is not a string");
        }
    }
}
