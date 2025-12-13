use super::PivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(PivotLibrary)]
pub struct PivotLibraryFake;

impl PivotLibrary for PivotLibraryFake {
    fn reverse_shell_pty(&self, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }

    fn reverse_shell_repl(&self) -> Result<(), String> {
        Ok(())
    }

    fn ssh_exec(
        &self,
        _target: String,
        _port: i64,
        _command: String,
        _username: String,
        _password: Option<String>,
        _key: Option<String>,
        _key_password: Option<String>,
        _timeout: Option<i64>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("status".into(), Value::Int(0));
        map.insert("stdout".into(), Value::String("fake output".to_string()));
        map.insert("stderr".into(), Value::String("".to_string()));
        Ok(map)
    }

    fn ssh_copy(
        &self,
        _target: String,
        _port: i64,
        _src: String,
        _dst: String,
        _username: String,
        _password: Option<String>,
        _key: Option<String>,
        _key_password: Option<String>,
        _timeout: Option<i64>,
    ) -> Result<String, String> {
        Ok("Success".into())
    }

    fn port_scan(
        &self,
        _target_cidrs: Vec<String>,
        _ports: Vec<i64>,
        _protocol: String,
        _timeout: i64,
        _fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut map = BTreeMap::new();
        map.insert("ip".into(), Value::String("127.0.0.1".to_string()));
        map.insert("port".into(), Value::Int(80));
        map.insert("protocol".into(), Value::String("tcp".to_string()));
        map.insert("status".into(), Value::String("open".to_string()));
        Ok(vec![map])
    }

    fn arp_scan(&self, _target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut map = BTreeMap::new();
        map.insert("ip".into(), Value::String("192.168.1.1".to_string()));
        map.insert("mac".into(), Value::String("00:00:00:00:00:00".to_string()));
        map.insert("interface".into(), Value::String("eth0".to_string()));
        Ok(vec![map])
    }

    fn ncat(
        &self,
        _address: String,
        _port: i64,
        _data: String,
        _protocol: String,
    ) -> Result<String, String> {
        Ok("fake response".into())
    }
}
