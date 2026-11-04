use super::SshLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};

// ──────────────────────────────────────────────────────────────────────────────
// Fake ssh session handle (returned as Foreign Value)
// ──────────────────────────────────────────────────────────────────────────────

#[eldritch_library("ssh_session")]
pub trait FakeSshSessionLibrary {
    #[eldritch_method]
    fn exec(&self, _command: String) -> Result<BTreeMap<String, Value>, String>;
    #[eldritch_method]
    fn close(&self) -> Result<(), String>;
}

#[derive(Debug)]
#[eldritch_library_impl(FakeSshSessionLibrary)]
pub struct FakeSshSessionHandle {
    closed: spin::Mutex<bool>,
}

impl FakeSshSessionLibrary for FakeSshSessionHandle {
    fn exec(&self, _command: String) -> Result<BTreeMap<String, Value>, String> {
        if *self.closed.lock() {
            return Err("ssh_session is closed".to_string());
        }
        let mut map = BTreeMap::new();
        map.insert("status".into(), Value::Int(0));
        map.insert("stdout".into(), Value::String("fake output".to_string()));
        map.insert("stderr".into(), Value::String("".to_string()));
        Ok(map)
    }

    fn close(&self) -> Result<(), String> {
        *self.closed.lock() = true;
        Ok(())
    }
}

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

    fn session(
        &self,
        _target: String,
        _username: String,
        _password: Option<String>,
        _key: Option<String>,
        _key_password: Option<String>,
        _port: Option<i64>,
        _timeout: Option<i64>,
    ) -> Result<Value, String> {
        let handle = FakeSshSessionHandle {
            closed: spin::Mutex::new(false),
        };
        Ok(Value::Foreign(Arc::new(handle)))
    }
}
