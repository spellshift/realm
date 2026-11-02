use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

#[cfg(unix)]
pub fn uname() -> Result<BTreeMap<String, String>, String> {
    let mut uts: libc::utsname = unsafe { core::mem::zeroed() };
    let res = unsafe { libc::uname(&mut uts) };
    if res != 0 {
        return Err(alloc::format!(
            "uname syscall failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    unsafe fn cstr_to_string(ptr: *const libc::c_char) -> String {
        unsafe { core::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }

    let mut map = BTreeMap::new();
    unsafe {
        map.insert("sysname".to_string(), cstr_to_string(uts.sysname.as_ptr()));
        map.insert(
            "nodename".to_string(),
            cstr_to_string(uts.nodename.as_ptr()),
        );
        map.insert("release".to_string(), cstr_to_string(uts.release.as_ptr()));
        map.insert("version".to_string(), cstr_to_string(uts.version.as_ptr()));
        map.insert("machine".to_string(), cstr_to_string(uts.machine.as_ptr()));
        #[cfg(target_os = "linux")]
        map.insert(
            "domainname".to_string(),
            cstr_to_string(uts.domainname.as_ptr()),
        );
    }
    Ok(map)
}

#[cfg(not(unix))]
pub fn uname() -> Result<BTreeMap<String, String>, String> {
    Err(
        "This OS isn't supported by the uname function. Only linux systems are supported"
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_linux_uname() {
        let info = uname().unwrap();
        assert!(info.contains_key("sysname"));
        assert!(info.contains_key("nodename"));
        assert!(info.contains_key("release"));
        assert!(info.contains_key("version"));
        assert!(info.contains_key("machine"));
        #[cfg(target_os = "linux")]
        {
            assert_eq!(info.get("sysname").unwrap(), "Linux");
            assert!(info.contains_key("domainname"));
        }
    }

    #[test]
    #[cfg(not(unix))]
    fn test_linux_uname_non_unix() {
        assert!(uname().is_err());
    }
}
