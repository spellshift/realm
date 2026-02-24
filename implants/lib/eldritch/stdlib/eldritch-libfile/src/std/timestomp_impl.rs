#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::{Context, Result as AnyhowResult};
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn timestomp(
    path: String,
    mtime: Option<Value>,
    atime: Option<Value>,
    ctime: Option<Value>,
    ref_file: Option<String>,
) -> Result<(), String> {
    timestomp_impl(path, mtime, atime, ctime, ref_file).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn timestomp(
    _path: alloc::string::String,
    _mtime: Option<eldritch_core::Value>,
    _atime: Option<eldritch_core::Value>,
    _ctime: Option<eldritch_core::Value>,
    _ref_file: Option<alloc::string::String>,
) -> Result<(), alloc::string::String> {
    Err("timestomp requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn timestomp_impl(
    path: String,
    mtime: Option<Value>,
    atime: Option<Value>,
    ctime: Option<Value>,
    ref_file: Option<String>,
) -> AnyhowResult<()> {
    use std::fs;

    let mut final_mtime: Option<::std::time::SystemTime> = None;
    let mut final_atime: Option<::std::time::SystemTime> = None;
    let mut final_ctime: Option<::std::time::SystemTime> = None;

    // 1. If ref_file is provided, read its times
    if let Some(ref_path) = ref_file {
        let meta = fs::metadata(&ref_path).context("Failed to stat ref_file")?;
        final_mtime = meta.modified().ok();
        final_atime = meta.accessed().ok();
        final_ctime = meta.created().ok();
    }

    // 2. Parse explicit times (override ref_file)
    if let Some(m) = mtime {
        final_mtime = Some(parse_time(m)?);
    }
    if let Some(a) = atime {
        final_atime = Some(parse_time(a)?);
    }
    if let Some(c) = ctime {
        final_ctime = Some(parse_time(c)?);
    }

    // 3. Platform specific apply
    apply_timestamps(&path, final_mtime, final_atime, final_ctime)
}

#[cfg(feature = "stdlib")]
fn parse_time(val: Value) -> AnyhowResult<::std::time::SystemTime> {
    match val {
        Value::Int(i) => {
            // Epoch seconds
            Ok(::std::time::UNIX_EPOCH + ::std::time::Duration::from_secs(i as u64))
        }
        Value::String(s) => {
            // Try ISO 8601 first
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                return Ok(dt.into());
            }
            // Try naive? chrono supports various.
            // Let's also support a simpler format "YYYY-MM-DD HH:MM:SS"
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                // Assume local? No, let's assume UTC for consistency unless offset provided
                return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    dt,
                    chrono::Utc,
                )
                .into());
            }
            // Try just YYYY-MM-DD
            if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                let dt = d.and_hms_opt(0, 0, 0).unwrap();
                return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    dt,
                    chrono::Utc,
                )
                .into());
            }

            anyhow::bail!("Failed to parse date string: {}", s)
        }
        _ => anyhow::bail!("Invalid time value type (must be Int or String)"),
    }
}

#[cfg(all(unix, not(target_os = "solaris"), feature = "stdlib"))]
fn apply_timestamps(
    path: &str,
    mtime: Option<::std::time::SystemTime>,
    atime: Option<::std::time::SystemTime>,
    _ctime: Option<::std::time::SystemTime>, // Not supported on standard Linux
) -> AnyhowResult<()> {
    use nix::sys::time::TimeVal;
    use std::fs;

    // We need both atime and mtime for utimes. If one is missing, we should probably read the current one?
    // Or if ref_file wasn't provided, use current time?
    // The spec "touch" logic: if only one specified, update that one.
    // `utimensat` allows UTIME_OMIT to ignore. But `nix` wrapper might differ.

    // Let's get current times if needed
    let meta = fs::metadata(path).context("Failed to stat target file")?;

    // Helper to convert SystemTime to TimeVal
    fn system_time_to_timeval(t: ::std::time::SystemTime) -> TimeVal {
        let d = t
            .duration_since(::std::time::UNIX_EPOCH)
            .unwrap_or(::std::time::Duration::ZERO);
        TimeVal::new(
            d.as_secs() as i64,
            d.subsec_micros() as nix::libc::suseconds_t,
        )
    }

    let a_tv = if let Some(a) = atime {
        system_time_to_timeval(a)
    } else {
        meta.accessed()
            .ok()
            .map(system_time_to_timeval)
            .unwrap_or_else(|| {
                // Fallback if accessed() not supported? Use current time or 0?
                // Using current time is safer than 0.
                system_time_to_timeval(::std::time::SystemTime::now())
            })
    };

    let m_tv = if let Some(m) = mtime {
        system_time_to_timeval(m)
    } else {
        meta.modified()
            .ok()
            .map(system_time_to_timeval)
            .unwrap_or_else(|| system_time_to_timeval(::std::time::SystemTime::now()))
    };

    nix::sys::stat::utimes(path, &a_tv, &m_tv).context("Failed to set file times (utimes)")?;

    // ctime is ignored/unsupported
    Ok(())
}

#[cfg(all(windows, feature = "stdlib"))]
fn apply_timestamps(
    path: &str,
    mtime: Option<::std::time::SystemTime>,
    atime: Option<::std::time::SystemTime>,
    ctime: Option<::std::time::SystemTime>,
) -> AnyhowResult<()> {
    use std::fs::OpenOptions;
    use windows_sys::Win32::Foundation::{FILETIME, HANDLE};
    use windows_sys::Win32::Storage::FileSystem::SetFileTime;

    // We need to open the file handle
    // windows-sys takes raw pointers.
    // Convert path to wide string (actually windows-sys might prefer wide or ANSI depending on fn. SetFileTime needs Handle. CreateFileW needs wide.)
    // Wait, I imported CreateFileA (ANSI). Let's use W.

    // Actually, std::fs::File can give us the handle?
    // std::fs::File::open might not give WRITE_ATTRIBUTES access if we just open for read/write?
    // Let's use OpenOptions to open for write?
    // fs::OpenOptions doesn't expose FILE_WRITE_ATTRIBUTES directly.
    // Calling `SetFileTime` requires a handle with `FILE_WRITE_ATTRIBUTES`.
    // Opening with `write(true)` gives `GENERIC_WRITE` which includes `FILE_WRITE_ATTRIBUTES`.

    let file = OpenOptions::new().write(true).open(path)?;
    use std::os::windows::io::AsRawHandle;
    let handle = file.as_raw_handle() as HANDLE;

    fn to_filetime(t: ::std::time::SystemTime) -> FILETIME {
        let since_epoch = t
            .duration_since(::std::time::UNIX_EPOCH)
            .unwrap_or(::std::time::Duration::ZERO);
        // Windows epoch is 1601-01-01. Unix is 1970-01-01.
        // Difference is 11644473600 seconds.
        // Ticks are 100ns.
        let intervals = since_epoch.as_nanos() / 100 + 116444736000000000;
        FILETIME {
            dwLowDateTime: (intervals & 0xFFFFFFFF) as u32,
            dwHighDateTime: (intervals >> 32) as u32,
        }
    }

    let ft_creation = ctime.map(to_filetime);
    let ft_access = atime.map(to_filetime);
    let ft_write = mtime.map(to_filetime);

    let p_creation = ft_creation
        .as_ref()
        .map(|p| p as *const FILETIME)
        .unwrap_or(std::ptr::null());
    let p_access = ft_access
        .as_ref()
        .map(|p| p as *const FILETIME)
        .unwrap_or(std::ptr::null());
    let p_write = ft_write
        .as_ref()
        .map(|p| p as *const FILETIME)
        .unwrap_or(std::ptr::null());

    let res = unsafe { SetFileTime(handle, p_creation, p_access, p_write) };

    if res == 0 {
        anyhow::bail!("SetFileTime failed");
    }

    Ok(())
}

#[cfg(not(any(all(unix, not(target_os = "solaris")), windows)))]
#[cfg(feature = "stdlib")]
fn apply_timestamps(
    _path: &str,
    _mtime: Option<::std::time::SystemTime>,
    _atime: Option<::std::time::SystemTime>,
    _ctime: Option<::std::time::SystemTime>,
) -> AnyhowResult<()> {
    anyhow::bail!("timestomp not supported on this platform")
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_timestomp() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Initial stat
        let initial_meta = fs::metadata(&path).unwrap();
        let _initial_mtime = initial_meta.modified().unwrap();

        // Wait a bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_secs(1));

        // 1. Set specific mtime (Int)
        let target_time = std::time::SystemTime::now();
        let target_secs = target_time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        timestomp(
            path.clone(),
            Some(Value::Int(target_secs as i64)),
            None,
            None,
            None,
        )
        .unwrap();

        let new_meta = fs::metadata(&path).unwrap();
        let new_mtime = new_meta.modified().unwrap();
        let new_secs = new_mtime
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Allow small skew if file system has low resolution (e.g. some only support seconds)
        // But since we use same secs, should be equal or close.
        assert!((new_secs as i64 - target_secs as i64).abs() <= 1);

        // 2. Set specific atime (String)
        let time_str = "2020-01-01 12:00:00"; // UTC implied
        timestomp(
            path.clone(),
            None,
            Some(Value::String(time_str.to_string())),
            None,
            None,
        )
        .unwrap();

        let meta2 = fs::metadata(&path).unwrap();
        let atime2 = meta2.accessed().unwrap();
        let atime_secs = atime2
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 2020-01-01 12:00:00 UTC = 1577880000
        assert_eq!(atime_secs, 1577880000);

        // 3. Ref file
        let ref_tmp = NamedTempFile::new().unwrap();
        let ref_path = ref_tmp.path().to_string_lossy().to_string();

        // Set ref file time to something old
        // Actually we can't easily set ref file time without using our own lib,
        // but we can just use the current time of ref file (which is fresh)
        // vs the target file which we just set to 2020.

        // Wait and touch ref file (re-write)
        std::thread::sleep(std::time::Duration::from_secs(1));
        fs::write(&ref_path, "update").unwrap();
        let ref_meta = fs::metadata(&ref_path).unwrap();
        let ref_mtime = ref_meta
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        timestomp(path.clone(), None, None, None, Some(ref_path)).unwrap();

        let final_meta = fs::metadata(&path).unwrap();
        let final_mtime = final_meta
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert_eq!(final_mtime, ref_mtime);
    }
}
