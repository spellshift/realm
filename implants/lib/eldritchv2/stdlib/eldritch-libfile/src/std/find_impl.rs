#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use std::fs;
#[cfg(feature = "stdlib")]
use std::path::Path;

#[cfg(feature = "stdlib")]
pub fn find(
    path: String,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> Result<Vec<String>, String> {
    find_impl(
        path,
        name,
        file_type,
        permissions,
        modified_time,
        create_time,
    )
    .map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn find(
    _path: alloc::string::String,
    _name: Option<alloc::string::String>,
    _file_type: Option<alloc::string::String>,
    _permissions: Option<i64>,
    _modified_time: Option<i64>,
    _create_time: Option<i64>,
) -> Result<Vec<alloc::string::String>, alloc::string::String> {
    Err("find requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn find_impl(
    path: String,
    name: Option<String>,
    file_type: Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<Vec<String>> {
    let mut out = Vec::new();
    let root = Path::new(&path);
    if !root.is_dir() {
        return Ok(out);
    }

    // Recursive search
    find_recursive(
        root,
        &mut out,
        &name,
        &file_type,
        permissions,
        modified_time,
        create_time,
    )?;

    Ok(out)
}

#[cfg(feature = "stdlib")]
fn find_recursive(
    dir: &Path,
    out: &mut Vec<String>,
    name: &Option<String>,
    file_type: &Option<String>,
    permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<()> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_recursive(
                    &path,
                    out,
                    name,
                    file_type,
                    permissions,
                    modified_time,
                    create_time,
                )?;
            }

            if check_path(
                &path,
                name,
                file_type,
                permissions,
                modified_time,
                create_time,
            )? {
                if let Ok(p) = path.canonicalize() {
                    out.push(p.to_string_lossy().to_string());
                } else {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "stdlib")]
fn check_path(
    path: &Path,
    name: &Option<String>,
    file_type: &Option<String>,
    _permissions: Option<i64>,
    modified_time: Option<i64>,
    create_time: Option<i64>,
) -> AnyhowResult<bool> {
    if let Some(n) = name {
        if let Some(fname) = path.file_name() {
            if !fname.to_string_lossy().contains(n) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }

    if let Some(ft) = file_type {
        if ft == "file" && !path.is_file() {
            return Ok(false);
        }
        if ft == "dir" && !path.is_dir() {
            return Ok(false);
        }
    }

    // Note: Permissions check on V1 was strict (==).
    #[cfg(unix)]
    if let Some(p) = _permissions {
        use ::std::os::unix::fs::PermissionsExt;
        let meta = path.metadata()?;
        if (meta.permissions().mode() & 0o777) as i64 != p {
            return Ok(false);
        }
    }

    if let Some(mt) = modified_time {
        let meta = path.metadata()?;
        if meta
            .modified()
            .and_then(|t| {
                t.duration_since(::std::time::UNIX_EPOCH)
                    .map_err(std::io::Error::other)
            })
            .map(|d| d.as_secs() as i64)
            .is_ok_and(|secs| secs != mt)
        {
            return Ok(false);
        }
    }

    if let Some(ct) = create_time {
        let meta = path.metadata()?;
        if meta
            .created()
            .and_then(|t| {
                t.duration_since(::std::time::UNIX_EPOCH)
                    .map_err(std::io::Error::other)
            })
            .map(|d| d.as_secs() as i64)
            .is_ok_and(|secs| secs != ct)
        {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[test]
    fn test_find() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let base_path = tmp_dir.path();

        // Setup directory structure
        let dir1 = base_path.join("dir1");
        fs::create_dir(&dir1).unwrap();
        let file1 = base_path.join("file1.txt");
        fs::write(&file1, "content1").unwrap();
        let file2 = dir1.join("file2.log");
        fs::write(&file2, "content2").unwrap();
        let file3 = dir1.join("file3.txt");
        fs::write(&file3, "content3").unwrap();

        let base_path_str = base_path.to_string_lossy().to_string();

        // 1. Basic list all
        let res = find(base_path_str.clone(), None, None, None, None, None).unwrap();
        // Should contain file1, file2, file3. Might contain dir1 too.
        // Logic says: `if path.is_dir() { recurse } if check_path() { push }`
        // check_path without filters returns true. So it should return dirs too.
        assert!(res.iter().any(|p| p.contains("file1.txt")));
        assert!(res.iter().any(|p| p.contains("file2.log")));
        assert!(res.iter().any(|p| p.contains("dir1")));

        // 2. Name filter
        let res = find(
            base_path_str.clone(),
            Some(".txt".to_string()),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(res.iter().any(|p| p.contains("file1.txt")));
        assert!(res.iter().any(|p| p.contains("file3.txt")));
        assert!(!res.iter().any(|p| p.contains("file2.log")));

        // 3. Type filter
        let res = find(
            base_path_str.clone(),
            None,
            Some("file".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        assert!(res.iter().all(|p| !Path::new(p).is_dir()));
        assert!(res.iter().any(|p| p.contains("file1.txt")));

        let res = find(
            base_path_str.clone(),
            None,
            Some("dir".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        assert!(res.iter().all(|p| Path::new(p).is_dir()));
        assert!(res.iter().any(|p| p.contains("dir1")));
    }
}
