#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::{Context, Result as AnyhowResult};
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn set_perms(
    path: String,
    user: Option<String>,
    group: Option<String>,
    perms: Option<String>,
    xattrs: Option<BTreeMap<String, Value>>,
) -> Result<(), String> {
    set_perms_impl(path, user, group, perms, xattrs).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn set_perms(
    _path: alloc::string::String,
    _user: Option<alloc::string::String>,
    _group: Option<alloc::string::String>,
    _perms: Option<alloc::string::String>,
    _xattrs: Option<alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>>,
) -> Result<(), alloc::string::String> {
    Err("set_perms requires stdlib feature".into())
}

#[cfg(all(feature = "stdlib", unix))]
fn set_perms_impl(
    path: String,
    user: Option<String>,
    group: Option<String>,
    perms: Option<String>,
    xattrs: Option<BTreeMap<String, Value>>,
) -> AnyhowResult<()> {
    use nix::unistd::{Gid, Group, Uid, User, chown};
    use std::os::unix::fs::PermissionsExt;

    // 1. Handle ownership
    let mut uid: Option<Uid> = None;
    if let Some(u) = user {
        if let Ok(id) = u.parse::<u32>() {
            uid = Some(Uid::from_raw(id));
        } else if let Ok(Some(user_info)) = User::from_name(&u) {
            uid = Some(user_info.uid);
        } else {
            return Err(anyhow::anyhow!("User not found: {}", u));
        }
    }

    let mut gid: Option<Gid> = None;
    if let Some(g) = group {
        if let Ok(id) = g.parse::<u32>() {
            gid = Some(Gid::from_raw(id));
        } else if let Ok(Some(group_info)) = Group::from_name(&g) {
            gid = Some(group_info.gid);
        } else {
            return Err(anyhow::anyhow!("Group not found: {}", g));
        }
    }

    if uid.is_some() || gid.is_some() {
        chown(path.as_str(), uid, gid).context("Failed to set ownership")?;
    }

    // 2. Handle permissions
    if let Some(p) = perms {
        let mode = u32::from_str_radix(&p, 8).context("Invalid octal permissions string")?;
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(&path, permissions).context("Failed to set permissions")?;
    }

    // 3. Handle extended attributes
    if let Some(attrs) = xattrs {
        for (key, value) in attrs {
            let bytes = match value {
                Value::String(s) => s.into_bytes(),
                Value::Bytes(b) => b,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Extended attribute value must be a string or bytes"
                    ));
                }
            };
            xattr::set(&path, &key, &bytes)
                .context(format!("Failed to set extended attribute: {}", key))?;
        }
    }

    Ok(())
}

#[cfg(all(feature = "stdlib", windows))]
fn set_perms_impl(
    path: String,
    user: Option<String>,
    group: Option<String>,
    perms: Option<String>,
    xattrs: Option<BTreeMap<String, Value>>,
) -> AnyhowResult<()> {
    if user.is_some() || group.is_some() {
        return Err(anyhow::anyhow!(
            "Ownership changes are not supported on Windows"
        ));
    }

    if let Some(attrs) = xattrs {
        if !attrs.is_empty() {
            return Err(anyhow::anyhow!(
                "Extended attributes are not supported on Windows"
            ));
        }
    }

    if let Some(p) = perms {
        // Just support read-only flag like standard rust Windows permissions
        let mode = u32::from_str_radix(&p, 8).context("Invalid octal permissions string")?;

        let mut permissions = std::fs::metadata(&path)
            .context("Failed to get file metadata")?
            .permissions();

        // In Windows, rust fs::Permissions only supports readonly
        // If mode is read only (e.g. missing write bit like 0444), we set readonly
        let readonly = (mode & 0o222) == 0;
        permissions.set_readonly(readonly);

        std::fs::set_permissions(&path, permissions).context("Failed to set permissions")?;
    }

    Ok(())
}

#[cfg(all(feature = "stdlib", not(any(unix, windows))))]
fn set_perms_impl(
    _path: String,
    _user: Option<String>,
    _group: Option<String>,
    _perms: Option<String>,
    _xattrs: Option<BTreeMap<String, Value>>,
) -> AnyhowResult<()> {
    Err(anyhow::anyhow!(
        "set_perms is not supported on this platform"
    ))
}

#[cfg(all(test, feature = "stdlib", unix))]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_set_perms() -> AnyhowResult<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"test")?;
        let path = file.path().to_string_lossy().to_string();

        // 1. Set permissions
        set_perms_impl(path.clone(), None, None, Some("755".to_string()), None)?;

        let metadata = std::fs::metadata(&path)?;
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o755);

        // 2. Set xattrs
        let mut xattrs = BTreeMap::new();
        xattrs.insert("user.test".to_string(), Value::String("value".to_string()));

        set_perms_impl(path.clone(), None, None, None, Some(xattrs))?;

        let attr = xattr::get(&path, "user.test")?;
        assert_eq!(attr, Some(b"value".to_vec()));

        Ok(())
    }
}

#[cfg(all(test, feature = "stdlib", windows))]
mod windows_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_set_perms_windows() -> AnyhowResult<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"test")?;
        let path = file.path().to_string_lossy().to_string();

        // 1. Setting permissions to readonly (no write bit)
        set_perms_impl(path.clone(), None, None, Some("444".to_string()), None)?;

        let metadata = std::fs::metadata(&path)?;
        assert_eq!(metadata.permissions().readonly(), true);

        // 2. Setting permissions to read-write
        set_perms_impl(path.clone(), None, None, Some("644".to_string()), None)?;

        let metadata = std::fs::metadata(&path)?;
        assert_eq!(metadata.permissions().readonly(), false);

        // 3. Ownership should error
        let res = set_perms_impl(path.clone(), Some("user".to_string()), None, None, None);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Ownership changes are not supported on Windows"
        );

        // 4. Extended attributes should error
        let mut xattrs = BTreeMap::new();
        xattrs.insert("user.test".to_string(), Value::String("value".to_string()));
        let res = set_perms_impl(path.clone(), None, None, None, Some(xattrs));
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Extended attributes are not supported on Windows"
        );

        Ok(())
    }
}
