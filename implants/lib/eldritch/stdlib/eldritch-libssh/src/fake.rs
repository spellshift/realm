use super::SshLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(SshLibrary)]
pub struct SshLibraryFake;

impl SshLibrary for SshLibraryFake {
    fn copy(
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

    fn deploy(
        &self,
        _ips: Vec<String>,
        _credentials: Vec<BTreeMap<String, Value>>,
        _cmd: String,
        _privesc_cmd: Option<String>,
        _payload: Option<Vec<u8>>,
        _payload_dst: Option<String>,
        _timeout: Option<i64>,
        _retries: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut map = BTreeMap::new();
        map.insert("ip".into(), Value::String("127.0.0.1".to_string()));
        map.insert("status".into(), Value::String("success".to_string()));
        map.insert("principal".into(), Value::String("root".to_string()));
        map.insert("stdout".into(), Value::String("fake output".to_string()));
        map.insert("stderr".into(), Value::String("".to_string()));
        map.insert("error".into(), Value::String("".to_string()));
        Ok(vec![map])
    }

    fn exec(
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
}
