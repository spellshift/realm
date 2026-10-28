use super::ScanLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(ScanLibrary)]
pub struct ScanLibraryFake;

impl ScanLibrary for ScanLibraryFake {
    fn arp(&self, _target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut map = BTreeMap::new();
        map.insert("ip".into(), Value::String("192.168.1.1".to_string()));
        map.insert("mac".into(), Value::String("00:00:00:00:00:00".to_string()));
        map.insert("interface".into(), Value::String("eth0".to_string()));
        Ok(vec![map])
    }

    fn ports(
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
}
