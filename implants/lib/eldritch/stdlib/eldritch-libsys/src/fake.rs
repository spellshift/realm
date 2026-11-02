use super::SysLibrary;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_libfile::{FileLibrary, fake::FileLibraryFake};
use eldritch_libnet::{NetLibrary, fake::NetLibraryFake};
use eldritch_libprocess::{ProcessLibrary, fake::ProcessLibraryFake};
use eldritch_libregistry::{RegistryLibrary, fake::RegistryLibraryFake};
use eldritch_libuser::{UserLibrary, fake::UserLibraryFake};
use eldritch_libwindows::{WindowsLibrary, fake::WindowsLibraryFake};
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(SysLibrary)]
pub struct SysLibraryFake;

impl SysLibrary for SysLibraryFake {
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String> {
        WindowsLibraryFake.dll_inject(dll_path, pid)
    }

    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String> {
        WindowsLibraryFake.dll_reflect(dll_bytes, pid, function_name)
    }

    fn exec(
        &self,
        _path: String,
        _args: Vec<String>,
        _disown: Option<bool>,
        _env_vars: Option<BTreeMap<String, String>>,
        _input: Option<String>,
    ) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_env(&self) -> Result<BTreeMap<String, String>, String> {
        ProcessLibraryFake.env()
    }

    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        NetLibraryFake.interfaces()
    }

    fn get_os(&self) -> Result<BTreeMap<String, String>, String> {
        self.info()
    }

    fn get_pid(&self) -> Result<i64, String> {
        ProcessLibraryFake.pid()
    }

    fn get_reg(&self, path: String) -> Result<BTreeMap<String, String>, String> {
        RegistryLibraryFake.read(path)
    }

    fn get_user(&self) -> Result<BTreeMap<String, Value>, String> {
        UserLibraryFake.info()
    }

    fn hostname(&self) -> Result<String, String> {
        Ok(String::from("eldritch-test-box"))
    }

    fn impersonate(&self, pid: i64) -> Result<i64, String> {
        WindowsLibraryFake.impersonate(pid)
    }

    fn info(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("os".into(), "linux".into());
        map.insert("arch".into(), "x86_64".into());
        map.insert("version".into(), "5.4.0-generic".into());
        map.insert("platform".into(), "PLATFORM_LINUX".into());
        Ok(map)
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

    fn list_users(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        UserLibraryFake.list()
    }

    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("stdout".into(), Value::String(format!("Executed: {}", cmd)));
        map.insert("stderr".into(), Value::String("".into()));
        map.insert("status".into(), Value::Int(0));
        Ok(map)
    }

    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        WindowsLibraryFake.tokens(pid)
    }

    fn use_token(&self, id: i64) -> Result<bool, String> {
        WindowsLibraryFake.use_token(id)
    }

    fn whereis(&self, bin: String) -> Result<String, String> {
        FileLibraryFake::default().which(bin)
    }

    fn write_reg(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String> {
        RegistryLibraryFake.write(path, regname, regtype, regvalue)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_sys_fake() {
        let sys = SysLibraryFake;
        assert_eq!(sys.get_pid().unwrap(), 1337);
        assert!(sys.is_linux().unwrap());
        assert!(!sys.is_windows().unwrap());
        assert_eq!(sys.hostname().unwrap(), "eldritch-test-box");
        assert!(sys.get_env().unwrap().contains_key("PATH"));
        assert!(sys.info().unwrap().contains_key("platform"));
    }
}
