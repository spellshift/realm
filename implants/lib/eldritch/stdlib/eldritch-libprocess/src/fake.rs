use super::ProcessLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct ProcessLibraryFake;

impl ProcessLibrary for ProcessLibraryFake {
    fn info(&self, _pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("cmd".into(), Value::String("target/debug/golem -i".into()));
        map.insert(
            "cwd".into(),
            Value::String("/workspaces/realm/implants".into()),
        );

        map.insert(
            "exe".into(),
            Value::String("/workspaces/realm/implants/target/debug/golem".into()),
        );
        map.insert("gid".into(), Value::Int(1001));
        map.insert("memory_usage".into(), Value::Int(16384000));
        map.insert("name".into(), Value::String("golem".into()));
        map.insert("pid".into(), Value::Int(151931));
        map.insert("ppid".into(), Value::Int(76290));
        map.insert("root".into(), Value::String("/".into()));
        map.insert("run_time".into(), Value::Int(139));
        map.insert("start_time".into(), Value::Int(1769925749));
        map.insert("status".into(), Value::String("Runnable".into()));
        map.insert("uid".into(), Value::Int(1000));
        map.insert("virtual_memory_usage".into(), Value::Int(37724160));
        Ok(map)
    }

    fn kill(&self, _pid: i64) -> Result<(), String> {
        Ok(())
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut p1 = BTreeMap::new();
        p1.insert("name".into(), Value::String("init".into()));
        p1.insert("pid".into(), Value::Int(1));
        p1.insert("ppid".into(), Value::Int(0));
        p1.insert("arch".into(), Value::String("x86_64".into()));
        p1.insert("principal".into(), Value::String("root".into()));
        p1.insert("command".into(), Value::String("/sbin/init".into()));

        let mut p2 = BTreeMap::new();
        p2.insert("name".into(), Value::String("bash".into()));
        p2.insert("pid".into(), Value::Int(1001));
        p2.insert("ppid".into(), Value::Int(1));
        p2.insert("arch".into(), Value::String("x86_64".into()));
        p2.insert("principal".into(), Value::String("user".into()));
        p2.insert("command".into(), Value::String("/bin/bash".into()));

        let mut p3 = BTreeMap::new();
        p3.insert("name".into(), Value::String("eldritch".into()));
        p3.insert("pid".into(), Value::Int(1337)); // The PID returned by netstat
        p3.insert("ppid".into(), Value::Int(1));
        p3.insert("arch".into(), Value::String("x86_64".into()));
        p3.insert("principal".into(), Value::String("user".into()));
        p3.insert("command".into(), Value::String("./eldritch".into()));

        Ok(vec![p1, p2, p3])
    }

    fn name(&self, _pid: i64) -> Result<String, String> {
        Ok("fake-process".into())
    }

    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut conn = BTreeMap::new();
        conn.insert("protocol".into(), Value::String("tcp".into()));
        conn.insert("local_address".into(), Value::String("127.0.0.1".into()));
        conn.insert("local_port".into(), Value::Int(80));
        conn.insert("remote_address".into(), Value::String("0.0.0.0".into()));
        conn.insert("remote_port".into(), Value::Int(0));
        conn.insert("state".into(), Value::String("LISTEN".into()));
        conn.insert("pid".into(), Value::Int(1337));
        conn.insert("socket_type".into(), Value::String("STREAM".into()));
        Ok(vec![conn])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_process_list() {
        let lib = ProcessLibraryFake;
        let list = lib.list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].get("name"), Some(&Value::String("init".into())));
        assert_eq!(
            list[0].get("principal"),
            Some(&Value::String("root".into()))
        );
        assert_eq!(list[1].get("name"), Some(&Value::String("bash".into())));
        assert_eq!(list[2].get("name"), Some(&Value::String("eldritch".into())));
    }

    #[test]
    fn test_fake_process_info() {
        let lib = ProcessLibraryFake;
        let info = lib.info(Some(123)).unwrap(); // PID doesn't matter for fake
        assert_eq!(info.get("name"), Some(&Value::String("golem".into())));
        assert_eq!(info.get("pid"), Some(&Value::Int(151931)));
    }

    #[test]
    fn test_fake_process_name() {
        let lib = ProcessLibraryFake;
        let name = lib.name(123).unwrap();
        assert_eq!(name, "fake-process");
    }

    #[test]
    fn test_fake_process_kill() {
        let lib = ProcessLibraryFake;
        // Should always succeed
        assert!(lib.kill(123).is_ok());
    }

    #[test]
    fn test_fake_process_netstat() {
        let lib = ProcessLibraryFake;
        let netstat = lib.netstat().unwrap();
        assert_eq!(netstat.len(), 1);
        assert_eq!(netstat[0].get("local_port"), Some(&Value::Int(80)));
    }
}
