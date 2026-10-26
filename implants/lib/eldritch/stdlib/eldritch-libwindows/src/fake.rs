use super::WindowsLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(WindowsLibrary)]
pub struct WindowsLibraryFake;

impl WindowsLibrary for WindowsLibraryFake {
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

    fn impersonate(&self, _pid: i64) -> Result<i64, String> {
        Ok(1)
    }

    fn read_reg(&self, _path: String) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn tokens(&self, _pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut entry = BTreeMap::new();
        entry.insert("id".into(), Value::Int(1));
        entry.insert(
            "source".into(),
            Value::String("impersonate:explorer.exe".into()),
        );
        entry.insert("active".into(), Value::Bool(true));
        Ok(vec![entry])
    }

    fn use_token(&self, _id: i64) -> Result<bool, String> {
        Ok(true)
    }

    fn write_reg(
        &self,
        _path: String,
        _regname: String,
        _regtype: String,
        _regvalue: Value,
    ) -> Result<bool, String> {
        Ok(true)
    }
}
