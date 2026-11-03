use super::LinuxLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use eldritch_macros::eldritch_library_impl;
use spin::Mutex;

#[derive(Debug)]
#[eldritch_library_impl(LinuxLibrary)]
pub struct LinuxLibraryFake {
    attrs: Mutex<BTreeMap<String, String>>,
}

impl Default for LinuxLibraryFake {
    fn default() -> Self {
        Self {
            attrs: Mutex::new(BTreeMap::new()),
        }
    }
}

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

    fn chattr(&self, path: String, attrs: String) -> Result<(), String> {
        let mut op = attrs.chars();
        let Some(mode) = op.next() else {
            return Err("attrs must not be empty".to_string());
        };
        if !matches!(mode, '+' | '-' | '=') {
            return Err("attrs must start with '+', '-', or '='".to_string());
        }
        let requested: String = op.collect();

        let mut store = self.attrs.lock();
        let current = store.entry(path).or_default();
        match mode {
            '=' => *current = requested,
            '+' => {
                for c in requested.chars() {
                    if !current.contains(c) {
                        current.push(c);
                    }
                }
            }
            '-' => current.retain(|c| !requested.contains(c)),
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_linux_fake() {
        let linux = LinuxLibraryFake::default();
        let info = linux.uname().unwrap();
        assert_eq!(info.get("sysname").unwrap(), "Linux");
        assert_eq!(info.get("machine").unwrap(), "x86_64");
    }

    #[test]
    fn test_linux_fake_chattr() {
        let linux = LinuxLibraryFake::default();
        linux
            .chattr("/tmp/foo".to_string(), "+i".to_string())
            .unwrap();
        assert_eq!(linux.attrs.lock().get("/tmp/foo").unwrap(), "i");

        linux
            .chattr("/tmp/foo".to_string(), "+a".to_string())
            .unwrap();
        let val = linux.attrs.lock().get("/tmp/foo").unwrap().clone();
        assert!(val.contains('i') && val.contains('a'));

        linux
            .chattr("/tmp/foo".to_string(), "-i".to_string())
            .unwrap();
        assert_eq!(linux.attrs.lock().get("/tmp/foo").unwrap(), "a");

        assert!(
            linux
                .chattr("/tmp/foo".to_string(), "i".to_string())
                .is_err()
        );
    }
}
