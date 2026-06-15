#[cfg(feature = "stdlib")]
use alloc::string::String;

#[cfg(feature = "stdlib")]
pub fn set_perms(path: String, mode: i64) -> Result<(), String> {
    set_perms_impl(&path, mode).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn set_perms(_path: String, _mode: i64) -> Result<(), String> {
    Err("set_perms requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn set_perms_impl(path: &str, mode: i64) -> Result<(), String> {
    use std::fs;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Parse mode as octal if it looks like an octal number
        // Accept both 0o755 (already parsed as decimal 493) and 755 (should be treated as octal)
        let mode_u32 = mode as u32;

        // Check if mode is already in decimal representation of octal
        // e.g., 0o755 = 493 in decimal, so if mode is 755, interpret as octal
        let final_mode = if mode_u32 > 0o7777 {
            // Already decimal representation of octal (like 493 for 0o755)
            mode_u32 & 0o7777
        } else {
            // Treat as octal literal (like 755 -> 0o755)
            // Parse the mode as if it were written in octal
            let mode_str = mode.to_string();
            u32::from_str_radix(&mode_str, 8)
                .map_err(|_| format!("Invalid octal mode: {}", mode))?
                & 0o7777
        };

        let metadata = fs::metadata(path).map_err(|e| format!("Cannot access '{}': {}", path, e))?;
        let mut perms = metadata.permissions();
        perms.set_mode(final_mode);
        fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set permissions on '{}': {}", path, e))?;

        Ok(())
    }

    #[cfg(not(unix))]
    {
        // On Windows, only support read-only (mode > 0) or writable (mode == 0)
        let metadata = fs::metadata(path).map_err(|e| format!("Cannot access '{}': {}", path, e))?;
        let mut perms = metadata.permissions();
        perms.set_readonly(mode != 0);
        fs::set_permissions(path, perms)
            .map_err(|e| format!("Failed to set permissions on '{}': {}", path, e))?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use crate::std::get_perms_impl;
    use tempfile::NamedTempFile;

    #[cfg(unix)]
    #[test]
    fn test_set_perms_unix() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Set to 0o644
        set_perms(path.clone(), 644).unwrap();
        let perms = get_perms_impl::get_perms(path.clone()).unwrap();
        assert_eq!(perms, "0644");

        // Set to 0o755
        set_perms(path.clone(), 755).unwrap();
        let perms = get_perms_impl::get_perms(path.clone()).unwrap();
        assert_eq!(perms, "0755");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_set_perms_windows() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Set to read-only
        set_perms(path.clone(), 1).unwrap();
        let perms = get_perms_impl::get_perms(path.clone()).unwrap();
        assert_eq!(perms, "r");

        // Set to writable
        set_perms(path.clone(), 0).unwrap();
        let perms = get_perms_impl::get_perms(path.clone()).unwrap();
        assert_eq!(perms, "rw");
    }

    #[test]
    fn test_set_perms_nonexistent() {
        let result = set_perms("/nonexistent/path/that/does/not/exist".to_string(), 644);
        assert!(result.is_err());
    }
}
