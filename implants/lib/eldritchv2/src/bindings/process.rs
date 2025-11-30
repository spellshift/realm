use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[eldritch_library("process")]
pub trait ProcessLibrary {
    #[eldritch_method]
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn kill(&self, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn name(&self, pid: i64) -> Result<String, String>;

    #[eldritch_method]
    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct ProcessLibraryFake;

#[cfg(feature = "fake_bindings")]
impl ProcessLibrary for ProcessLibraryFake {
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("pid".into(), Value::Int(pid.unwrap_or(123)));
        map.insert("name".into(), Value::String("fake_proc".into()));
        Ok(map)
    }

    fn kill(&self, _pid: i64) -> Result<(), String> { Ok(()) }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut map = BTreeMap::new();
        map.insert("pid".into(), Value::Int(123));
        map.insert("name".into(), Value::String("fake_proc".into()));
        Ok(vec![map])
    }

    fn name(&self, _pid: i64) -> Result<String, String> {
        Ok(String::from("fake_proc"))
    }

    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Ok(Vec::new())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_process_fake() {
        let process = ProcessLibraryFake::default();
        let _info = process.info(Some(999)).unwrap();
        assert!(!process.list().unwrap().is_empty());
        assert_eq!(process.name(123).unwrap(), "fake_proc");
    }
}
