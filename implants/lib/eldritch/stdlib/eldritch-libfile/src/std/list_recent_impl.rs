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
pub fn list_recent(path: String, limit: i64) -> Result<Vec<String>, String> {
    let mut entries = Vec::new();
    let root = Path::new(&path);

    if !root.exists() {
        return Err(alloc::format!("Path does not exist: {}", path));
    }

    visit_dirs(root, &mut entries).map_err(|e| e.to_string())?;

    // Sort by modified time descending
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));

    // Take limit
    let limit = if limit < 0 { 0 } else { limit as usize };
    let result = entries.into_iter().take(limit).map(|e| e.path).collect();

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
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
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
    }
    Ok(())
}

#[cfg(not(feature = "stdlib"))]
pub fn list_recent(_path: String, _limit: i64) -> Result<Vec<String>, String> {
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
        let res = list_recent(base_path_str.clone(), 1).unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].contains("file3.txt"));

        // Test limit 2 (should be file3, file2)
        let res = list_recent(base_path_str.clone(), 2).unwrap();
        assert_eq!(res.len(), 2);
        assert!(res[0].contains("file3.txt"));
        assert!(res[1].contains("file2.txt"));

        // Test limit 3 (should be file3, file2, file1)
        let res = list_recent(base_path_str.clone(), 10).unwrap();
        assert_eq!(res.len(), 3);
        assert!(res[0].contains("file3.txt"));
        assert!(res[1].contains("file2.txt"));
        assert!(res[2].contains("file1.txt"));
    }

    #[test]
    fn test_list_recent_empty() {
        let tmp_dir = TempDir::new().unwrap();
        let base_path_str = tmp_dir.path().to_string_lossy().to_string();

        let res = list_recent(base_path_str, 10).unwrap();
        assert!(res.is_empty());
    }
}
