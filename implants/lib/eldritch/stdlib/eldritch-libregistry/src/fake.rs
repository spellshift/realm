use crate::RegistryLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(RegistryLibrary)]
pub struct RegistryLibraryFake;

impl RegistryLibrary for RegistryLibraryFake {
    fn read(&self, _path: String) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("DummyValue".into(), "DummyData".into());
        Ok(map)
    }

    fn write(
        &self,
        _path: String,
        _regname: String,
        _regtype: String,
        _regvalue: Value,
    ) -> Result<bool, String> {
        Ok(true)
    }
}
