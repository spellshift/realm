use alloc::string::String;
use alloc::string::ToString;

pub fn chmod(path: String, mode: i64) -> Result<(), String> {
    chmod_impl(path, mode as u32)
}

#[cfg(unix)]
fn chmod_impl(path: String, mode: u32) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_mode(mode);
    fs::set_permissions(&path, perms).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn chmod_impl(path: String, mode: u32) -> Result<(), String> {
    use std::fs;

    let mut perms = fs::metadata(&path)
        .map_err(|e| e.to_string())?
        .permissions();

    // Windows logic: only check the 0o200 bit (owner writable)
    let is_writable = (mode & 0o200) != 0;
    perms.set_readonly(!is_writable);

    fs::set_permissions(&path, perms).map_err(|e| e.to_string())
}

#[cfg(not(any(unix, windows)))]
fn chmod_impl(_path: String, _mode: u32) -> Result<(), String> {
    Err("chmod is not supported on this platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg(unix)]
    fn test_chmod_unix() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        chmod(path.clone(), 0o755).unwrap();

        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }

    #[test]
    #[cfg(windows)]
    fn test_chmod_windows() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        // Try setting writable (clear readonly)
        chmod(path.clone(), 0o600).unwrap();
        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.readonly(), false);

        // Try setting readonly
        chmod(path.clone(), 0o400).unwrap();
        let perms = fs::metadata(&path).unwrap().permissions();
        assert_eq!(perms.readonly(), true);
    }
}
