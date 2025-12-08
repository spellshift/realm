
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use spin::Mutex;
use super::ProcessLibrary;

#[derive(Clone, Debug)]
struct ProcessInfo {
    pid: i64,
    ppid: i64,
    name: String,
    user: String,
    arch: String,
}

#[derive(Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct ProcessLibraryFake {
    processes: Arc<Mutex<BTreeMap<i64, ProcessInfo>>>,
}

impl Default for ProcessLibraryFake {
    fn default() -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            1,
            ProcessInfo {
                pid: 1,
                ppid: 0,
                name: "init".to_string(),
                user: "root".to_string(),
                arch: "x86_64".to_string(),
            },
        );
        map.insert(
            2,
            ProcessInfo {
                pid: 2,
                ppid: 0,
                name: "kthreadd".to_string(),
                user: "root".to_string(),
                arch: "x86_64".to_string(),
            },
        );
        map.insert(
            100,
            ProcessInfo {
                pid: 100,
                ppid: 1,
                name: "systemd".to_string(),
                user: "root".to_string(),
                arch: "x86_64".to_string(),
            },
        );
        map.insert(
            1000,
            ProcessInfo {
                pid: 1000,
                ppid: 100,
                name: "sshd".to_string(),
                user: "root".to_string(),
                arch: "x86_64".to_string(),
            },
        );
        map.insert(
            1001,
            ProcessInfo {
                pid: 1001,
                ppid: 1000,
                name: "bash".to_string(),
                user: "user".to_string(),
                arch: "x86_64".to_string(),
            },
        );
        map.insert(
            1337,
            ProcessInfo {
                pid: 1337,
                ppid: 1001,
                name: "eldritch".to_string(),
                user: "user".to_string(),
                arch: "x86_64".to_string(),
            },
        );

        Self {
            processes: Arc::new(Mutex::new(map)),
        }
    }
}

impl ProcessLibrary for ProcessLibraryFake {
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
        let pid = pid.unwrap_or(1337); // Default to current process
        let procs = self.processes.lock();
        if let Some(p) = procs.get(&pid) {
            let mut map = BTreeMap::new();
            map.insert("pid".to_string(), Value::Int(p.pid));
            map.insert("ppid".to_string(), Value::Int(p.ppid));
            map.insert("name".to_string(), Value::String(p.name.clone()));
            map.insert("user".to_string(), Value::String(p.user.clone()));
            map.insert("arch".to_string(), Value::String(p.arch.clone()));
            Ok(map)
        } else {
            Err("Process not found".to_string())
        }
    }

    fn kill(&self, pid: i64) -> Result<(), String> {
        let mut procs = self.processes.lock();
        if procs.remove(&pid).is_some() {
            Ok(())
        } else {
            Err("Process not found".to_string())
        }
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let procs = self.processes.lock();
        let mut list = Vec::new();
        for p in procs.values() {
            let mut map = BTreeMap::new();
            map.insert("pid".to_string(), Value::Int(p.pid));
            map.insert("ppid".to_string(), Value::Int(p.ppid));
            map.insert("name".to_string(), Value::String(p.name.clone()));
            map.insert("user".to_string(), Value::String(p.user.clone()));
            map.insert("arch".to_string(), Value::String(p.arch.clone()));
            list.push(map);
        }
        Ok(list)
    }

    fn name(&self, pid: i64) -> Result<String, String> {
        let procs = self.processes.lock();
        if let Some(p) = procs.get(&pid) {
            Ok(p.name.clone())
        } else {
            Err("Process not found".to_string())
        }
    }

    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Ok(Vec::new())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::ProcessLibraryFake;
    use crate::ProcessLibrary;

    #[test]
    fn test_process_fake() {
        let process = ProcessLibraryFake::default();
        let list = process.list().unwrap();
        assert!(list.len() >= 5);

        let info = process.info(Some(1001)).unwrap();
        assert_eq!(info.get("name").unwrap().to_string(), "bash");

        process.kill(1001).unwrap();
        assert!(process.info(Some(1001)).is_err());
    }
}
