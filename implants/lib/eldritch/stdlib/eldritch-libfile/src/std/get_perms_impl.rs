#[cfg(feature = "stdlib")]
use alloc::string::String;

#[cfg(feature = "stdlib")]
pub fn get_perms(path: String) -> Result<String, String> {
    get_perms_impl(&path).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn get_perms(_path: String) -> Result<String, String> {
    Err("get_perms requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn get_perms_impl(path: &str) -> Result<String, String> {
    use std::fs;

    let metadata = fs::metadata(path).map_err(|e| format!("Cannot access '{}': {}", path, e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        // Return the last 3 octal digits (rwxrwxrwx)
        Ok(format!("{:04o}", mode & 0o7777))
    }

    #[cfg(not(unix))]
    {
        // On Windows, return "r" for read-only, "rw" for writable
        if metadata.permissions().readonly() {
            Ok("r".to_string())
        } else {
            Ok("rw".to_string())
        }
    }
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[cfg(unix)]
    #[test]
    fn test_get_perms_unix() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let perms = get_perms(path.clone()).unwrap();
        assert!(perms.len() >= 3);

        // Should be a valid octal number
        let _mode: u32 = u32::from_str_radix(&perms, 8).unwrap();
    }

    #[cfg(not(unix))]
    #[test]
    fn test_get_perms_windows() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let perms = get_perms(path).unwrap();
        assert!(perms == "r" || perms == "rw");
    }

    #[test]
    fn test_get_perms_nonexistent() {
        let result = get_perms("/nonexistent/path/that/does/not/exist".to_string());
        assert!(result.is_err());
    }
}
