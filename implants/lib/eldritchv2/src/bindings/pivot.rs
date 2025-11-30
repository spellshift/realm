use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[eldritch_library("pivot")]
pub trait PivotLibrary {
    #[eldritch_method]
    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    fn bind_proxy(&self, listen_address: String, listen_port: i64, username: String, password: String) -> Result<(), String>;

    #[eldritch_method]
    fn ncat(&self, address: String, port: i64, data: String, protocol: String) -> Result<String, String>;

    #[eldritch_method]
    fn port_forward(&self, listen_address: String, listen_port: i64, forward_address: String, forward_port: i64, protocol: String) -> Result<(), String>;

    #[eldritch_method]
    fn port_scan(&self, target_cidrs: Vec<String>, ports: Vec<i64>, protocol: String, timeout: i64) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    fn smb_exec(&self, target: String, port: i64, username: String, password: String, hash: String, command: String) -> Result<String, String>;

    #[eldritch_method]
    fn ssh_copy(&self, target: String, port: i64, src: String, dst: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<i64>) -> Result<String, String>;

    #[eldritch_method]
    fn ssh_exec(&self, target: String, port: i64, command: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(PivotLibrary)]
pub struct PivotLibraryFake;

#[cfg(feature = "fake_bindings")]
impl PivotLibrary for PivotLibraryFake {
    fn arp_scan(&self, _target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, String>>, String> {
        Ok(Vec::new())
    }

    fn bind_proxy(&self, _listen_address: String, _listen_port: i64, _username: String, _password: String) -> Result<(), String> { Ok(()) }

    fn ncat(&self, _address: String, _port: i64, _data: String, _protocol: String) -> Result<String, String> {
        Ok(String::new())
    }

    fn port_forward(&self, _listen_address: String, _listen_port: i64, _forward_address: String, _forward_port: i64, _protocol: String) -> Result<(), String> { Ok(()) }

    fn port_scan(&self, _target_cidrs: Vec<String>, _ports: Vec<i64>, _protocol: String, _timeout: i64) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Ok(Vec::new())
    }

    fn reverse_shell_pty(&self, _cmd: Option<String>) -> Result<(), String> { Ok(()) }

    fn smb_exec(&self, _target: String, _port: i64, _username: String, _password: String, _hash: String, _command: String) -> Result<String, String> {
        Ok(String::new())
    }

    fn ssh_copy(&self, _target: String, _port: i64, _src: String, _dst: String, _username: String, _password: Option<String>, _key: Option<String>, _key_password: Option<String>, _timeout: Option<i64>) -> Result<String, String> {
        Ok(String::from("Success"))
    }

    fn ssh_exec(&self, _target: String, _port: i64, _command: String, _username: String, _password: Option<String>, _key: Option<String>, _key_password: Option<String>, _timeout: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Ok(Vec::new())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_fake() {
        let pivot = PivotLibraryFake::default();
        assert!(pivot.arp_scan(vec![]).unwrap().is_empty());
        pivot.bind_proxy("127.0.0.1".into(), 8080, "user".into(), "pass".into()).unwrap();
        assert_eq!(pivot.ssh_copy("t".into(), 22, "s".into(), "d".into(), "u".into(), None, None, None, None).unwrap(), "Success");
    }
}
