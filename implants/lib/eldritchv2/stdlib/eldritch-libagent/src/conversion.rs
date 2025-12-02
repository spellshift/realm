#[cfg(feature = "stdlib")]
use alloc::format;
#[cfg(feature = "stdlib")]
use alloc::string::{String, ToString};
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;
#[cfg(feature = "stdlib")]
use eldritch_core::conversion::{FromValue, ToValue};
#[cfg(feature = "stdlib")]
use pb::c2;
#[cfg(feature = "stdlib")]
use pb::eldritch;

// --- Wrappers ---

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct TaskWrapper(pub c2::Task);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct TaskOutputWrapper(pub c2::TaskOutput);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct AgentWrapper(pub c2::Agent);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct BeaconWrapper(pub c2::Beacon);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct HostWrapper(pub c2::Host);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct CredentialWrapper(pub eldritch::Credential);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct FileWrapper(pub eldritch::File);

#[cfg(feature = "stdlib")]
#[derive(Debug, Clone)]
pub struct ProcessListWrapper(pub eldritch::ProcessList);

// --- ToValue Implementations (Returning to Eldritch) ---

#[cfg(feature = "stdlib")]
impl ToValue for TaskWrapper {
    fn to_value(self) -> Value {
        let task = self.0;
        let mut map = BTreeMap::new();
        map.insert("id".to_string(), Value::Int(task.id));
        map.insert("quest_name".to_string(), Value::String(task.quest_name));
        // Tome is complex, let's represent it as a dict or None for now
        // For strict correctness we might want a TomeWrapper, but often scripts just need the ID.
        // If needed, we can expand Tome.
        Value::Dictionary(alloc::rc::Rc::new(core::cell::RefCell::new(map)))
    }
}

// NOTE: impl ToValue for Vec<T> is provided by eldritch_core::conversion,
// so we do not implement it for Vec<TaskWrapper> here.

// --- FromValue Implementations (Arguments from Eldritch) ---

#[cfg(feature = "stdlib")]
impl FromValue for CredentialWrapper {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                // pb::eldritch::Credential fields: principal, secret, kind
                let principal = dict.get("principal")
                    .or_else(|| dict.get("user")) // alias
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let secret = dict.get("secret")
                    .or_else(|| dict.get("password")) // alias
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                // Ignoring Kind for now, default to Unspecified (0)

                Ok(CredentialWrapper(eldritch::Credential {
                    principal,
                    secret,
                    kind: 0,
                }))
            }
            _ => Err(format!("Expected Dictionary for Credential, got {:?}", v))
        }
    }
}

#[cfg(feature = "stdlib")]
impl FromValue for FileWrapper {
    fn from_value(v: &Value) -> Result<Self, String> {
         match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                let path = dict.get("path").map(|v| v.to_string()).unwrap_or_default();
                let chunk = if let Some(Value::Bytes(b)) = dict.get("content") {
                    b.clone()
                } else {
                    Vec::new()
                };

                // Construct FileMetadata
                let metadata = eldritch::FileMetadata {
                    path,
                    // other metadata fields ignored/defaulted
                    ..Default::default()
                };

                Ok(FileWrapper(eldritch::File {
                    metadata: Some(metadata),
                    chunk,
                }))
            }
            _ => Err(format!("Expected Dictionary for File, got {:?}", v))
        }
    }
}

#[cfg(feature = "stdlib")]
impl FromValue for ProcessListWrapper {
    fn from_value(v: &Value) -> Result<Self, String> {
         // ProcessList has `repeated Process list`.
         match v {
             Value::List(l) => {
                 let list_val = l.borrow();
                 let mut processes = Vec::new();
                 for item in list_val.iter() {
                     // Assume item is a dict representing a Process
                     if let Value::Dictionary(d) = item {
                         let d = d.borrow();
                         let pid = d.get("pid").and_then(|v| match v { Value::Int(i) => Some(*i as u64), _ => None }).unwrap_or(0);
                         let name = d.get("name").map(|v| v.to_string()).unwrap_or_default();
                         // ... other fields
                         processes.push(eldritch::Process {
                             pid,
                             name,
                             ..Default::default()
                         });
                     }
                 }
                 Ok(ProcessListWrapper(eldritch::ProcessList { list: processes }))
             }
             _ => Err(format!("Expected List for ProcessList, got {:?}", v))
         }
    }
}

#[cfg(feature = "stdlib")]
impl FromValue for TaskOutputWrapper {
    fn from_value(_v: &Value) -> Result<Self, String> {
        // This might not be needed if we use simple args for report_task_output
        Err("TaskOutputWrapper FromValue not implemented".to_string())
    }
}

// Helpers for responses
#[cfg(feature = "stdlib")]
impl ToValue for CredentialWrapper { // Not needed usually for return
    fn to_value(self) -> Value { Value::None }
}
