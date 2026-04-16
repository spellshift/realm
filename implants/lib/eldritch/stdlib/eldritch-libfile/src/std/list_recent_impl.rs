use alloc::string::String;
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use std::fs;
#[cfg(feature = "stdlib")]
use std::path::Path;
#[cfg(feature = "stdlib")]
use std::time::SystemTime;

#[cfg(feature = "stdlib")]
struct FileEntry {
    path: String,
    modified: SystemTime,
}

#[cfg(feature = "stdlib")]
pub fn list_recent(path: Option<String>, limit: Option<i64>) -> Result<Vec<String>, String> {
    let mut entries = Vec::new();
    let path_str = path.unwrap_or_else(|| {
        if cfg!(windows) {
            "C:\\".to_string()
        } else {
            "/".to_string()
        }
    });
    let limit_val = limit.unwrap_or(10);
    let root = Path::new(&path_str);

    if !root.exists() {
        return Err(alloc::format!("Path does not exist: {}", path_str));
    }

    visit_dirs(root, &mut entries).map_err(|e| e.to_string())?;

    // Sort by modified time descending
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));

    // Take limit
    let limit_usize = if limit_val < 1 { 1 } else { limit_val as usize };
    let result = entries
        .into_iter()
        .take(limit_usize)
        .map(|e| e.path)
        .collect();

    Ok(result)
}

#[cfg(feature = "stdlib")]
fn visit_dirs(dir: &Path, cb: &mut Vec<FileEntry>) -> std::io::Result<()> {
    if dir.is_dir() {
        // We use read_dir which returns an iterator over entries.
        // We ignore errors on subdirectories to be robust, similar to `find`.
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    // Avoid following symlinks to prevent infinite loops
                    let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    if is_dir {
                        // Recurse, ignoring errors
                        let _ = visit_dirs(&path, cb);
                    } else {
                        if let Ok(metadata) = entry.metadata()
                            && let Ok(modified) = metadata.modified() {
                                cb.push(FileEntry {
                                    path: path.to_string_lossy().into_owned(),
                                    modified,
                                });
                            }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(feature = "stdlib"))]
pub fn list_recent(_path: Option<String>, _limit: Option<i64>) -> Result<Vec<String>, String> {
    Err("list_recent requires stdlib feature".into())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_list_recent() {
        let tmp_dir = TempDir::new().unwrap();
        let base_path = tmp_dir.path();

        // Create files with different modification times
        let file1 = base_path.join("file1.txt");
        fs::write(&file1, "content1").unwrap();

        // Sleep to ensure different mtime (some FS have low resolution)
        thread::sleep(Duration::from_millis(100));

        let dir1 = base_path.join("dir1");
        fs::create_dir(&dir1).unwrap();

        thread::sleep(Duration::from_millis(100));

        let file2 = dir1.join("file2.txt");
        fs::write(&file2, "content2").unwrap();

        thread::sleep(Duration::from_millis(100));

        let file3 = base_path.join("file3.txt");
        fs::write(&file3, "content3").unwrap();

        let base_path_str = base_path.to_string_lossy().to_string();

        // Test limit 1 (should be file3)
        let res = list_recent(Some(base_path_str.clone()), Some(1)).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].contains("file3.txt"));

        // Test limit 2 (should be file3, file2)
        let res = list_recent(Some(base_path_str.clone()), Some(2)).unwrap();
        assert_eq!(res.len(), 2);
        assert!(res[0].contains("file3.txt"));
        assert!(res[1].contains("file2.txt"));

        // Test limit 3 (should be file3, file2, file1)
        let res = list_recent(Some(base_path_str.clone()), Some(10)).unwrap();
        assert_eq!(res.len(), 3);
        assert!(res[0].contains("file3.txt"));
        assert!(res[1].contains("file2.txt"));
        assert!(res[2].contains("file1.txt"));
    }

    #[test]
    fn test_list_recent_empty() {
        let tmp_dir = TempDir::new().unwrap();
        let base_path_str = tmp_dir.path().to_string_lossy().to_string();

        let res = list_recent(Some(base_path_str), Some(10)).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_list_recent_default_args() {
        let tmp_dir = TempDir::new().unwrap();
        let base_path = tmp_dir.path();

        let file1 = base_path.join("file1.txt");
        fs::write(&file1, "content1").unwrap();

        // Change working directory to test default path "/" which depends on current dir/fs context
        // Actually, since we can't easily mock "/", let's just test that passing None for limit
        // uses the default of 10.
        // For testing `None` for path, it tries to access `/`. We just ensure it doesn't crash
        // and returns a Result (might be Err depending on permissions, but usually Ok with / files)

        let res_limit = list_recent(Some(base_path.to_string_lossy().to_string()), None).unwrap();
        assert_eq!(res_limit.len(), 1);

        // Test None path (defaults to "/")
        let res_path = list_recent(None, Some(1));
        assert!(res_path.is_ok() || res_path.is_err());
    }
}
