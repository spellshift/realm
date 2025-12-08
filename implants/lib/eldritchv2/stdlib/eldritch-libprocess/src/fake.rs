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
        map.insert("name".into(), Value::String("init".into()));
        map.insert("pid".into(), Value::Int(1));
        map.insert("ppid".into(), Value::Int(0));
        map.insert("arch".into(), Value::String("x86_64".into()));
        map.insert("user".into(), Value::String("root".into()));
        map.insert("command".into(), Value::String("/sbin/init".into()));
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
        p1.insert("user".into(), Value::String("root".into()));
        p1.insert("command".into(), Value::String("/sbin/init".into()));

        let mut p2 = BTreeMap::new();
        p2.insert("name".into(), Value::String("bash".into()));
        p2.insert("pid".into(), Value::Int(1001));
        p2.insert("ppid".into(), Value::Int(1));
        p2.insert("arch".into(), Value::String("x86_64".into()));
        p2.insert("user".into(), Value::String("user".into()));
        p2.insert("command".into(), Value::String("/bin/bash".into()));

        let mut p3 = BTreeMap::new();
        p3.insert("name".into(), Value::String("eldritch".into()));
        p3.insert("pid".into(), Value::Int(1337)); // The PID returned by netstat
        p3.insert("ppid".into(), Value::Int(1));
        p3.insert("arch".into(), Value::String("x86_64".into()));
        p3.insert("user".into(), Value::String("user".into()));
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
