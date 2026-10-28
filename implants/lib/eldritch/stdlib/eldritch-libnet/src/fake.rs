use super::NetLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(NetLibrary)]
pub struct NetLibraryFake;

impl NetLibrary for NetLibraryFake {
    fn interfaces(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        let mut net = BTreeMap::new();
        net.insert("name".into(), "eth0".into());
        net.insert("ip".into(), "192.168.1.100".into());

        let mut lo = BTreeMap::new();
        lo.insert("name".into(), "lo".into());
        lo.insert("ip".into(), "127.0.0.1".into());

        Ok(vec![net, lo])
    }

    fn conns(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
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

    fn tcp(&self, _address: String, _port: i64, _data: String) -> Result<String, String> {
        Ok("fake response".into())
    }

    fn udp(&self, _address: String, _port: i64, _data: String) -> Result<String, String> {
        Ok("fake response".into())
    }
}
