use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_core::conversion::{FromValue, ToValue};
use spin::RwLock;

#[derive(Debug, Clone)]
pub struct TaskWrapper;

#[derive(Debug, Clone)]
pub struct CredentialWrapper;

#[derive(Debug, Clone)]
pub struct FileWrapper;

#[derive(Debug, Clone)]
pub struct ProcessListWrapper;

impl FromValue for CredentialWrapper {
    fn from_value(_v: &Value) -> Result<Self, String> {
        Ok(CredentialWrapper)
    }
}

impl FromValue for FileWrapper {
    fn from_value(_v: &Value) -> Result<Self, String> {
        Ok(FileWrapper)
    }
}

impl FromValue for ProcessListWrapper {
    fn from_value(_v: &Value) -> Result<Self, String> {
        Ok(ProcessListWrapper)
    }
}

impl ToValue for TaskWrapper {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        map.insert(Value::String("id".to_string()), Value::Int(0));
        map.insert(
            Value::String("quest_name".to_string()),
            Value::String("fake".to_string()),
        );
        Value::Dictionary(Arc::new(RwLock::new(map)))
    }
}
