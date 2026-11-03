use alloc::format;
use alloc::string::String;

/// Parses a `chmod`-style mode string into a raw permission value.
///
/// Accepts plain octal (`"755"`, `"0755"`) or explicitly prefixed octal (`"0o755"`) strings.
fn parse_mode(mode: &str) -> Result<u32, String> {
    let trimmed = mode.trim();
    let digits = trimmed
        .strip_prefix("0o")
        .or_else(|| trimmed.strip_prefix("0O"))
        .unwrap_or(trimmed);

    if digits.is_empty() {
        return Err(format!("Invalid mode '{mode}': mode string is empty"));
    }

    u32::from_str_radix(digits, 8).map_err(|e| {
        format!("Invalid mode '{mode}': expected an octal permission string (e.g. \"0755\"): {e}")
    })
}

pub fn chmod(path: String, mode: String) -> Result<(), String> {
    let parsed_mode = parse_mode(&mode)?;
    set_permissions(&path, parsed_mode)
}

#[cfg(unix)]
fn set_permissions(path: &str, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, perms).map_err(|e| format!("Failed to chmod {path}: {e}"))
}

// On Windows, `os.File.Chmod` only honors the owner-writable bit (0o200): it toggles the
// file's read-only attribute, and all other bits are ignored. This mirrors that behavior.
#[cfg(windows)]
fn set_permissions(path: &str, mode: u32) -> Result<(), String> {
    let mut perms = std::fs::metadata(path)
        .map_err(|e| format!("Failed to stat {path}: {e}"))?
        .permissions();
    perms.set_readonly(mode & 0o200 == 0);
    std::fs::set_permissions(path, perms).map_err(|e| format!("Failed to chmod {path}: {e}"))
}

#[cfg(not(any(unix, windows)))]
fn set_permissions(_path: &str, _mode: u32) -> Result<(), String> {
    Err("file.chmod is not supported on this platform".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("755").unwrap(), 0o755);
        assert_eq!(parse_mode("0755").unwrap(), 0o755);
        assert_eq!(parse_mode("0o644").unwrap(), 0o644);
        assert_eq!(parse_mode("0O600").unwrap(), 0o600);
        assert_eq!(parse_mode(" 0755 ").unwrap(), 0o755);
    }

    #[test]
    fn test_parse_mode_errors() {
        assert!(parse_mode("").is_err());
        assert!(parse_mode("rwxr-xr-x").is_err());
        assert!(parse_mode("999").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_chmod_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        chmod(path.clone(), "0600".to_string()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        chmod(path.clone(), "0o755".to_string()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }

    #[cfg(unix)]
    #[test]
    fn test_chmod_invalid_mode() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        assert!(chmod(path, "not-a-mode".to_string()).is_err());
    }
}
