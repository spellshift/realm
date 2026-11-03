extern crate alloc;
use alloc::string::String;

/// Attribute table matching `chattr`'s settable flags (see e2fsprogs `misc/chattr.c`).
/// Order is not significant here since this is used purely for lookup.
#[cfg(target_os = "linux")]
const SETTABLE_FLAGS: &[(char, i64)] = &[
    ('A', 0x0000_0080), // No_Atime
    ('S', 0x0000_0008), // Synchronous_Updates
    ('D', 0x0001_0000), // Synchronous_Directory_Updates
    ('a', 0x0000_0020), // Append_Only
    ('c', 0x0000_0004), // Compression_Requested
    ('m', 0x0000_0400), // Dont_Compress
    ('d', 0x0000_0040), // No_Dump
    ('e', 0x0008_0000), // Extents
    ('i', 0x0000_0010), // Immutable
    ('j', 0x0000_4000), // Journaled_Data
    ('P', 0x2000_0000), // Project_Hierarchy
    ('s', 0x0000_0001), // Secure_Deletion
    ('u', 0x0000_0002), // Undelete
    ('t', 0x0000_8000), // No_Tailmerging
    ('T', 0x0002_0000), // Top_of_Directory_Hierarchies
    ('C', 0x0080_0000), // No_COW
    ('x', 0x0200_0000), // DAX
    ('F', 0x4000_0000), // Casefold
];

/// Full attribute table (in `lsattr` display order, see e2fsprogs `lib/e2p/pf.c`), used to
/// render a human-readable attribute string, including flags that cannot be set via `chattr`.
#[cfg(target_os = "linux")]
const DISPLAY_FLAGS: &[(char, i64)] = &[
    ('s', 0x0000_0001), // Secure_Deletion
    ('u', 0x0000_0002), // Undelete
    ('S', 0x0000_0008), // Synchronous_Updates
    ('D', 0x0001_0000), // Synchronous_Directory_Updates
    ('i', 0x0000_0010), // Immutable
    ('a', 0x0000_0020), // Append_Only
    ('d', 0x0000_0040), // No_Dump
    ('A', 0x0000_0080), // No_Atime
    ('c', 0x0000_0004), // Compression_Requested
    ('E', 0x0000_0800), // Encrypted
    ('j', 0x0000_4000), // Journaled_Data
    ('I', 0x0000_1000), // Indexed_directory
    ('t', 0x0000_8000), // No_Tailmerging
    ('T', 0x0002_0000), // Top_of_Directory_Hierarchies
    ('e', 0x0008_0000), // Extents
    ('C', 0x0080_0000), // No_COW
    ('x', 0x0200_0000), // DAX
    ('F', 0x4000_0000), // Casefold
    ('N', 0x1000_0000), // Inline_Data
    ('P', 0x2000_0000), // Project_Hierarchy
    ('V', 0x0010_0000), // Verity
    ('m', 0x0000_0400), // Dont_Compress
];

#[cfg(target_os = "linux")]
const FS_IOC_TYPE: libc::c_ulong = 0x66; // 'f'

#[cfg(target_os = "linux")]
fn ioc(dir: libc::c_ulong, nr: libc::c_ulong, size: usize) -> libc::c_ulong {
    (dir << 30) | (FS_IOC_TYPE << 8) | nr | ((size as libc::c_ulong) << 16)
}

#[cfg(target_os = "linux")]
fn get_flags(fd: core::ffi::c_int) -> Result<i64, String> {
    let mut flags: libc::c_long = 0;
    let req = ioc(2, 1, core::mem::size_of::<libc::c_long>());
    let ret = unsafe { libc::ioctl(fd, req as _, &mut flags) };
    if ret != 0 {
        return Err(alloc::format!(
            "Failed to read attributes: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(flags as i64)
}

#[cfg(target_os = "linux")]
fn set_flags(fd: core::ffi::c_int, flags: i64) -> Result<(), String> {
    let flags: libc::c_long = flags as libc::c_long;
    let req = ioc(1, 2, core::mem::size_of::<libc::c_long>());
    let ret = unsafe { libc::ioctl(fd, req as _, &flags) };
    if ret != 0 {
        return Err(alloc::format!(
            "Failed to set attributes: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn settable_bit(c: char) -> Option<i64> {
    SETTABLE_FLAGS
        .iter()
        .find(|(name, _)| *name == c)
        .map(|(_, bit)| *bit)
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct AttrOp {
    add: i64,
    rem: i64,
    set: Option<i64>,
}

#[cfg(target_os = "linux")]
fn parse_attrs(attrs: &str) -> Result<AttrOp, String> {
    let mut op = AttrOp::default();
    let mut mode: Option<char> = None;
    let mut set_bits: i64 = 0;

    for c in attrs.chars() {
        match c {
            '+' | '-' | '=' => {
                mode = Some(c);
                if c == '=' {
                    set_bits = 0;
                }
            }
            _ if c.is_whitespace() => continue,
            _ => {
                let Some(m) = mode else {
                    return Err(alloc::format!(
                        "Invalid attrs '{attrs}': must start with '+', '-', or '='"
                    ));
                };
                let bit = settable_bit(c).ok_or_else(|| {
                    alloc::format!("Unknown or unsupported attribute '{c}' in '{attrs}'")
                })?;
                match m {
                    '+' => op.add |= bit,
                    '-' => op.rem |= bit,
                    '=' => {
                        set_bits |= bit;
                        op.set = Some(set_bits);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    if mode.is_none() {
        return Err(alloc::format!(
            "Invalid attrs '{attrs}': must start with '+', '-', or '='"
        ));
    }

    Ok(op)
}

#[cfg(target_os = "linux")]
pub fn chattr(path: String, attrs: String) -> Result<(), String> {
    use std::os::unix::io::AsRawFd;

    let op = parse_attrs(&attrs)?;

    let file =
        std::fs::File::open(&path).map_err(|e| alloc::format!("Failed to open {path}: {e}"))?;
    let fd = file.as_raw_fd();
    let is_dir = file.metadata().map(|m| m.is_dir()).unwrap_or(false);

    let mut flags = get_flags(fd)?;
    if let Some(set_bits) = op.set {
        flags = set_bits;
    } else {
        flags &= !op.rem;
        flags |= op.add;
    }

    // `DIRSYNC` only applies to directories; mirror `chattr`'s behavior of stripping it
    // from non-directory targets.
    if !is_dir {
        flags &= !0x0001_0000;
    }

    set_flags(fd, flags)
}

#[cfg(not(target_os = "linux"))]
pub fn chattr(_path: String, _attrs: String) -> Result<(), String> {
    Err("linux.chattr is only supported on Linux".into())
}

/// Returns an `lsattr`-style attribute string (e.g. `"----i-------e--"`) for the file at `path`,
/// or `None` if the attributes could not be read (e.g. unsupported filesystem).
#[cfg(target_os = "linux")]
pub fn read_attributes(path: &str) -> Option<String> {
    use std::os::unix::io::AsRawFd;

    let file = std::fs::File::open(path).ok()?;
    let fd = file.as_raw_fd();
    let flags = get_flags(fd).ok()?;

    let mut out = String::with_capacity(DISPLAY_FLAGS.len());
    for (name, bit) in DISPLAY_FLAGS {
        if flags & *bit != 0 {
            out.push(*name);
        } else {
            out.push('-');
        }
    }
    Some(out)
}

#[cfg(not(target_os = "linux"))]
pub fn read_attributes(_path: &str) -> Option<String> {
    None
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attrs_add_rem_set() {
        let op = parse_attrs("+iA").unwrap();
        assert_eq!(op.add, 0x10 | 0x80);
        assert_eq!(op.rem, 0);
        assert_eq!(op.set, None);

        let op = parse_attrs("-i").unwrap();
        assert_eq!(op.rem, 0x10);

        let op = parse_attrs("=ia").unwrap();
        assert_eq!(op.set, Some(0x10 | 0x20));
    }

    #[test]
    fn test_parse_attrs_mixed() {
        let op = parse_attrs("+i-s").unwrap();
        assert_eq!(op.add, 0x10);
        assert_eq!(op.rem, 0x1);
    }

    #[test]
    fn test_parse_attrs_errors() {
        assert!(parse_attrs("i").is_err());
        assert!(parse_attrs("+z").is_err());
        assert!(parse_attrs("").is_err());
    }

    #[test]
    fn test_chattr_and_read_attributes_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Some container filesystems (e.g. overlayfs) don't support these ioctls; skip
        // gracefully if so, rather than failing the test on unsupported environments.
        if read_attributes(&path).is_none() {
            return;
        }

        chattr(path.clone(), "+A".to_string()).unwrap();
        let attrs = read_attributes(&path).unwrap();
        assert!(attrs.contains('A'));

        chattr(path.clone(), "-A".to_string()).unwrap();
        let attrs = read_attributes(&path).unwrap();
        assert!(!attrs.contains('A'));
    }
}
