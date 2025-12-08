use super::SysLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(SysLibrary)]
pub struct SysLibraryFake;

impl SysLibrary for SysLibraryFake {
    fn dll_inject(&self, _dll_path: String, _pid: i64) -> Result<(), String> {
        Ok(())
    }

    fn dll_reflect(
        &self,
        _dll_bytes: Vec<u8>,
        _pid: i64,
        _function_name: String,
    ) -> Result<(), String> {
        Ok(())
    }

    fn exec(
        &self,
        _path: String,
        _args: Vec<String>,
        _disown: Option<bool>,
        _env_vars: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_env(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("PATH".into(), "/usr/local/bin:/usr/bin:/bin".into());
        map.insert("HOME".into(), "/home/user".into());
        map.insert("USER".into(), "user".into());
        map.insert("TERM".into(), "xterm-256color".into());
        Ok(map)
    }

    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        let mut iface = BTreeMap::new();
        iface.insert("name".into(), "eth0".into());
        iface.insert("ip".into(), "192.168.1.100".into());
        Ok(vec![iface])
    }

    fn get_os(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("os".into(), "linux".into());
        map.insert("arch".into(), "x86_64".into());
        map.insert("version".into(), "5.4.0-generic".into());
        map.insert("platform".into(), "PLATFORM_LINUX".into()); // Fixed key value to match expected constant
        Ok(map)
    }

    fn get_pid(&self) -> Result<i64, String> {
        Ok(1337)
    }

    fn get_reg(
        &self,
        _reghive: String,
        _regpath: String,
    ) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn get_user(&self) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("username".into(), Value::String("root".into()));
        map.insert("uid".into(), Value::Int(0));
        map.insert("gid".into(), Value::Int(0));
        Ok(map)
    }

    fn hostname(&self) -> Result<String, String> {
        Ok(String::from("eldritch-test-box"))
    }

    fn is_bsd(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn is_linux(&self) -> Result<bool, String> {
        Ok(true)
    }

    fn is_macos(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn is_windows(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("stdout".into(), Value::String(format!("Executed: {}", cmd)));
        map.insert("stderr".into(), Value::String("".into()));
        map.insert("status".into(), Value::Int(0));
        Ok(map)
    }

    fn write_reg_hex(
        &self,
        _reghive: String,
        _regpath: String,
        _regname: String,
        _regtype: String,
        _regvalue: String,
    ) -> Result<bool, String> {
        Ok(true)
    }

    fn write_reg_int(
        &self,
        _reghive: String,
        _regpath: String,
        _regname: String,
        _regtype: String,
        _regvalue: i64,
    ) -> Result<bool, String> {
        Ok(true)
    }

    fn write_reg_str(
        &self,
        _reghive: String,
        _regpath: String,
        _regname: String,
        _regtype: String,
        _regvalue: String,
    ) -> Result<bool, String> {
        Ok(true)
    }
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
        assert_eq!(sys.hostname().unwrap(), "eldritch-test-box");
        assert!(sys.get_env().unwrap().contains_key("PATH"));
        assert!(sys.get_os().unwrap().contains_key("platform"));
    }
}
