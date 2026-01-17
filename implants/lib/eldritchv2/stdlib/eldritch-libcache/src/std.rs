use crate::CacheLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use spin::Mutex;

static CACHE: Mutex<BTreeMap<String, Value>> = Mutex::new(BTreeMap::new());

#[derive(Debug)]
#[eldritch_library_impl(CacheLibrary)]
pub struct StdCacheLibrary;

impl CacheLibrary for StdCacheLibrary {
    fn get(&self, key: String) -> Result<Value, String> {
        let guard = CACHE.lock();
        match guard.get(&key) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value::None),
        }
    }

    fn set(&self, key: String, val: Value) -> Result<(), String> {
        let mut guard = CACHE.lock();
        guard.insert(key, val);
        Ok(())
    }

    fn delete(&self, key: String) -> Result<Value, String> {
        let mut guard = CACHE.lock();
        match guard.remove(&key) {
            Some(v) => Ok(v),
            None => Ok(Value::None),
        }
    }
}