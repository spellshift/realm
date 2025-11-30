use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[eldritch_library("sys")]
pub trait SysLibrary {
    #[eldritch_method]
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    fn dll_reflect(&self, dll_bytes: Vec<u8>, pid: i64, function_name: String) -> Result<(), String>;

    #[eldritch_method]
    fn exec(&self, path: String, args: Vec<String>, disown: Option<bool>, env_vars: Option<BTreeMap<String, String>>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn get_env(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    fn get_os(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_pid(&self) -> Result<i64, String>;

    #[eldritch_method]
    fn get_reg(&self, reghive: String, regpath: String) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    fn get_user(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn hostname(&self) -> Result<String, String>;

    #[eldritch_method]
    fn is_bsd(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_linux(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_macos(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn is_windows(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn write_reg_hex(&self, reghive: String, regpath: String, regname: String, regtype: String, regvalue: String) -> Result<bool, String>;

    #[eldritch_method]
    fn write_reg_int(&self, reghive: String, regpath: String, regname: String, regtype: String, regvalue: i64) -> Result<bool, String>;

    #[eldritch_method]
    fn write_reg_str(&self, reghive: String, regpath: String, regname: String, regtype: String, regvalue: String) -> Result<bool, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(SysLibrary)]
pub struct SysLibraryFake;

#[cfg(feature = "fake_bindings")]
impl SysLibrary for SysLibraryFake {
    fn dll_inject(&self, _dll_path: String, _pid: i64) -> Result<(), String> { Ok(()) }

    fn dll_reflect(&self, _dll_bytes: Vec<u8>, _pid: i64, _function_name: String) -> Result<(), String> { Ok(()) }

    fn exec(&self, _path: String, _args: Vec<String>, _disown: Option<bool>, _env_vars: Option<BTreeMap<String, String>>) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_env(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        Ok(Vec::new())
    }

    fn get_os(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("os".into(), "linux".into());
        Ok(map)
    }

    fn get_pid(&self) -> Result<i64, String> {
        Ok(1337)
    }

    fn get_reg(&self, _reghive: String, _regpath: String) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn get_user(&self) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("username".into(), Value::String("root".into()));
        Ok(map)
    }

    fn hostname(&self) -> Result<String, String> {
        Ok(String::from("localhost"))
    }

    fn is_bsd(&self) -> Result<bool, String> { Ok(false) }

    fn is_linux(&self) -> Result<bool, String> { Ok(true) }

    fn is_macos(&self) -> Result<bool, String> { Ok(false) }

    fn is_windows(&self) -> Result<bool, String> { Ok(false) }

    fn shell(&self, _cmd: String) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn write_reg_hex(&self, _reghive: String, _regpath: String, _regname: String, _regtype: String, _regvalue: String) -> Result<bool, String> { Ok(true) }

    fn write_reg_int(&self, _reghive: String, _regpath: String, _regname: String, _regtype: String, _regvalue: i64) -> Result<bool, String> { Ok(true) }

    fn write_reg_str(&self, _reghive: String, _regpath: String, _regname: String, _regtype: String, _regvalue: String) -> Result<bool, String> { Ok(true) }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_sys_fake() {
        let sys = SysLibraryFake::default();
        assert_eq!(sys.get_pid().unwrap(), 1337);
        assert!(sys.is_linux().unwrap());
        assert!(!sys.is_windows().unwrap());
        assert_eq!(sys.hostname().unwrap(), "localhost");
    }
}
