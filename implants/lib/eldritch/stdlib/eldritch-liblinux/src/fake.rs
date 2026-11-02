use super::LinuxLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(LinuxLibrary)]
pub struct LinuxLibraryFake;

impl LinuxLibrary for LinuxLibraryFake {
    fn uname(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("sysname".into(), "Linux".into());
        map.insert("nodename".into(), "eldritch-test-box".into());
        map.insert("release".into(), "5.4.0-generic".into());
        map.insert("version".into(), "#1 SMP".into());
        map.insert("machine".into(), "x86_64".into());
        map.insert("domainname".into(), "(none)".into());
        Ok(map)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_linux_fake() {
        let linux = LinuxLibraryFake;
        let info = linux.uname().unwrap();
        assert_eq!(info.get("sysname").unwrap(), "Linux");
        assert_eq!(info.get("machine").unwrap(), "x86_64");
    }
}
