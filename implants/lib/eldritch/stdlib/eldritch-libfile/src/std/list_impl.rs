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
#[cfg(all(unix, not(target_os = "solaris")))]
use nix::unistd::{Gid, Group, Uid, User};
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
                // If I implement `handle_list` roughly:
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
    use alloc::format;

    let metadata = fs::metadata(path)?;
    let mut dict = BTreeMap::new();

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    dict.insert("file_name".to_string(), Value::String(name));

    let is_dir = metadata.is_dir();
    // Map to "file", "dir", "link", etc if possible.
    // V1 uses FileType enum.
    let type_str = if is_dir { "dir" } else { "file" }; // simplified
    dict.insert("type".to_string(), Value::String(type_str.to_string()));

    dict.insert("size".to_string(), Value::Int(metadata.len() as i64));

    // Permissions (simplified)
    #[cfg(unix)]
    use ::std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let perms = format!("{:o}", metadata.permissions().mode());
    #[cfg(not(unix))]
    let perms = if metadata.permissions().readonly() {
        "r"
    } else {
        "rw"
    }
    .to_string();

    dict.insert("permissions".to_string(), Value::String(perms));

    // Owner and Group
    #[cfg(all(unix, not(target_os = "solaris")))]
    {
        use ::std::os::unix::fs::MetadataExt;
        let uid = metadata.uid();
        let gid = metadata.gid();

        let user = User::from_uid(Uid::from_raw(uid)).ok().flatten();
        let group = Group::from_gid(Gid::from_raw(gid)).ok().flatten();

        let owner_name = user.map(|u| u.name).unwrap_or_else(|| uid.to_string());
        let group_name = group.map(|g| g.name).unwrap_or_else(|| gid.to_string());

        dict.insert("owner".to_string(), Value::String(owner_name));
        dict.insert("group".to_string(), Value::String(group_name));
    }
    #[cfg(any(not(unix), target_os = "solaris"))]
    {
        // Fallback for Windows or Solaris (without nix)
        #[cfg(target_os = "solaris")]
        {
            use ::std::os::unix::fs::MetadataExt;
            dict.insert(
                "owner".to_string(),
                Value::String(metadata.uid().to_string()),
            );
            dict.insert(
                "group".to_string(),
                Value::String(metadata.gid().to_string()),
            );
        }
        #[cfg(not(target_os = "solaris"))]
        {
            dict.insert("owner".to_string(), Value::String("".to_string()));
            dict.insert("group".to_string(), Value::String("".to_string()));
        }
    }

    // Absolute Path
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    dict.insert(
        "absolute_path".to_string(),
        Value::String(abs_path.to_string_lossy().to_string()),
    );

    // Times
    if let Ok(modified) = metadata.modified() {
        let dt: chrono::DateTime<chrono::Utc> = modified.into();
        let formatted = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        dict.insert("modified".to_string(), Value::String(formatted));
    }

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
