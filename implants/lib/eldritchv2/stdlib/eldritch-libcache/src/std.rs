use crate::CacheLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use spin::Mutex;

#[derive(Debug, Clone)]
#[eldritch_library_impl(CacheLibrary)]
pub struct StdCacheLibrary {
    store: Arc<Mutex<BTreeMap<String, Value>>>,
}

impl Default for StdCacheLibrary {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl StdCacheLibrary {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CacheLibrary for StdCacheLibrary {
    fn get(&self, key: String) -> Result<Value, String> {
        let guard = self.store.lock();
        match guard.get(&key) {
            Some(v) => Ok(v.clone()),
            None => Ok(Value::None),
        }
    }

    fn set(&self, key: String, val: Value) -> Result<(), String> {
        let mut guard = self.store.lock();
        guard.insert(key, val);
        Ok(())
    }

    fn delete(&self, key: String) -> Result<Value, String> {
        let mut guard = self.store.lock();
        match guard.remove(&key) {
            Some(v) => Ok(v),
            None => Ok(Value::None),
        }
    }
}
