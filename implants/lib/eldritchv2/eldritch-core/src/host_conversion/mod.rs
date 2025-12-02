use super::ast::Value;
use super::conversion::{FromValue, ToValue};
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use alloc::rc::Rc;
use eldritch_hostcontext::*;

// Implement FromValue for HostContext types

// Filesystem

impl ToValue for FileEntry {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("path".to_string(), Value::String(self.path));
        map.insert("name".to_string(), Value::String(self.name));
        map.insert("size".to_string(), Value::Int(self.size as i64));
        map.insert("is_dir".to_string(), Value::Bool(self.is_dir));
        map.insert("created".to_string(), Value::Int(self.created as i64));
        map.insert("modified".to_string(), Value::Int(self.modified as i64));
        map.insert("accessed".to_string(), Value::Int(self.accessed as i64));
        map.insert("mode".to_string(), Value::Int(self.mode as i64));
        Value::Dictionary(Rc::new(RefCell::new(map)))
    }
}

impl ToValue for ListDirResponse {
    fn to_value(self) -> Value {
        let entries: Vec<Value> = self.entries.into_iter().map(|e| e.to_value()).collect();
        Value::List(Rc::new(RefCell::new(entries)))
    }
}

impl ToValue for FileReadResponse {
    fn to_value(self) -> Value {
        Value::Bytes(self.data)
    }
}

impl ToValue for FileWriteResponse {
    fn to_value(self) -> Value {
        Value::Int(self.bytes_written as i64)
    }
}

impl ToValue for FileRemoveResponse {
    fn to_value(self) -> Value {
        Value::Bool(self.success)
    }
}

// Process

impl ToValue for ProcessInfo {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("pid".to_string(), Value::Int(self.pid as i64));
        map.insert("ppid".to_string(), Value::Int(self.ppid as i64));
        map.insert("name".to_string(), Value::String(self.name));
        map.insert("exe".to_string(), Value::String(self.exe));
        map.insert("cmdline".to_string(), Value::String(self.cmdline));
        map.insert("user".to_string(), Value::String(self.user));
        map.insert("start_time".to_string(), Value::Int(self.start_time as i64));
        map.insert("cpu_usage".to_string(), Value::Int(self.cpu_usage as i64));
        map.insert("memory_usage".to_string(), Value::Int(self.memory_usage as i64));
        Value::Dictionary(Rc::new(RefCell::new(map)))
    }
}

impl ToValue for ProcessListResponse {
    fn to_value(self) -> Value {
        let procs: Vec<Value> = self.processes.into_iter().map(|p| p.to_value()).collect();
        Value::List(Rc::new(RefCell::new(procs)))
    }
}

impl ToValue for ProcessKillResponse {
    fn to_value(self) -> Value {
        Value::Bool(self.success)
    }
}

impl ToValue for ExecResponse {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("pid".to_string(), Value::Int(self.pid as i64));
        map.insert("stdout".to_string(), Value::Bytes(self.stdout));
        map.insert("stderr".to_string(), Value::Bytes(self.stderr));
        map.insert("exit_code".to_string(), Value::Int(self.exit_code as i64));
        Value::Dictionary(Rc::new(RefCell::new(map)))
    }
}

// SysInfo

impl ToValue for SysInfoResponse {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("hostname".to_string(), Value::String(self.hostname));
        map.insert("os".to_string(), Value::String(self.os));
        map.insert("os_version".to_string(), Value::String(self.os_version));
        map.insert("kernel_version".to_string(), Value::String(self.kernel_version));
        map.insert("arch".to_string(), Value::String(self.arch));
        map.insert("num_cpus".to_string(), Value::Int(self.num_cpus as i64));
        map.insert("total_memory".to_string(), Value::Int(self.total_memory as i64));

        let ips: Vec<Value> = self.ip_addresses.into_iter().map(Value::String).collect();
        map.insert("ip_addresses".to_string(), Value::List(Rc::new(RefCell::new(ips))));

        map.insert("username".to_string(), Value::String(self.username));
        Value::Dictionary(Rc::new(RefCell::new(map)))
    }
}

// Env

impl ToValue for EnvGetResponse {
    fn to_value(self) -> Value {
        if self.found {
            Value::String(self.value)
        } else {
            Value::None
        }
    }
}

impl ToValue for EnvSetResponse {
    fn to_value(self) -> Value {
        Value::Bool(self.success)
    }
}

// Helpers for FromValue (Requests)

// ListDirRequest
impl FromValue for ListDirRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        // Can be initialized from a string path directly
        if let Value::String(s) = v {
            return Ok(ListDirRequest { path: s.clone() });
        }
        // Or a dictionary
        match v {
             Value::Dictionary(d) => {
                let dict = d.borrow();
                Ok(ListDirRequest {
                    path: extract_string(dict.get("path").ok_or("Missing path")?)?,
                })
             },
             _ => Err("Expected String or Dictionary for ListDirRequest".to_string())
        }
    }
}

// FileReadRequest
impl FromValue for FileReadRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                let path = extract_string(dict.get("path").ok_or("Missing path")?)?;

                let offset = match dict.get("offset") {
                    Some(val) => extract_u64(val)?,
                    None => 0,
                };

                let limit = match dict.get("limit") {
                    Some(val) => extract_u64(val)?,
                    None => 0,
                };

                Ok(FileReadRequest {
                     path,
                     offset,
                     limit,
                })
            },
             _ => Err("Expected Dictionary for FileReadRequest".to_string())
        }
    }
}

// FileWriteRequest
impl FromValue for FileWriteRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                let path = extract_string(dict.get("path").ok_or("Missing path")?)?;
                let data = extract_bytes(dict.get("data").ok_or("Missing data")?)?;
                let append = match dict.get("append") {
                    Some(val) => extract_bool(val)?,
                    None => false,
                };

                Ok(FileWriteRequest {
                    path,
                    data,
                    append,
                })
            },
            _ => Err("Expected Dictionary for FileWriteRequest".to_string())
        }
    }
}

// FileRemoveRequest
impl FromValue for FileRemoveRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
         if let Value::String(s) = v {
            return Ok(FileRemoveRequest { path: s.clone() });
        }
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                Ok(FileRemoveRequest {
                    path: extract_string(dict.get("path").ok_or("Missing path")?)?,
                })
            },
            _ => Err("Expected String or Dictionary for FileRemoveRequest".to_string())
        }
    }
}

// ProcessListRequest
impl FromValue for ProcessListRequest {
    fn from_value(_v: &Value) -> Result<Self, String> {
        Ok(ProcessListRequest {})
    }
}

// ProcessKillRequest
impl FromValue for ProcessKillRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Int(pid) => Ok(ProcessKillRequest { pid: *pid as u32 }),
            Value::Dictionary(d) => {
                let dict = d.borrow();
                Ok(ProcessKillRequest {
                    pid: extract_u32(dict.get("pid").ok_or("Missing pid")?)?,
                })
            }
            _ => Err("Expected Int or Dictionary for ProcessKillRequest".to_string())
        }
    }
}

// ExecRequest
impl FromValue for ExecRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                let path = extract_string(dict.get("path").ok_or("Missing path")?)?;

                let args = match dict.get("args") {
                    Some(val) => extract_string_list(val)?,
                    None => Vec::new(),
                };

                let env = match dict.get("env") {
                    Some(val) => extract_string_list(val)?,
                    None => Vec::new(),
                };

                let cwd = match dict.get("cwd") {
                    Some(val) => extract_string(val)?,
                    None => String::new(),
                };

                let background = match dict.get("background") {
                    Some(val) => extract_bool(val)?,
                    None => false,
                };

                Ok(ExecRequest {
                    path,
                    args,
                    env,
                    cwd,
                    background,
                })
            },
            _ => Err("Expected Dictionary for ExecRequest".to_string())
        }
    }
}

// SysInfoRequest
impl FromValue for SysInfoRequest {
    fn from_value(_v: &Value) -> Result<Self, String> {
        Ok(SysInfoRequest {})
    }
}

// EnvGetRequest
impl FromValue for EnvGetRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        if let Value::String(s) = v {
            return Ok(EnvGetRequest { key: s.clone() });
        }
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                Ok(EnvGetRequest {
                    key: extract_string(dict.get("key").ok_or("Missing key")?)?,
                })
            },
            _ => Err("Expected String or Dictionary for EnvGetRequest".to_string())
        }
    }
}

// EnvSetRequest
impl FromValue for EnvSetRequest {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                Ok(EnvSetRequest {
                    key: extract_string(dict.get("key").ok_or("Missing key")?)?,
                    value: extract_string(dict.get("value").ok_or("Missing value")?)?,
                })
            },
            _ => Err("Expected Dictionary for EnvSetRequest".to_string())
        }
    }
}


// ... helper extract functions ...
fn extract_string(v: &Value) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err("Expected String".to_string()),
    }
}

fn extract_u64(v: &Value) -> Result<u64, String> {
    match v {
        Value::Int(i) => Ok(*i as u64),
        _ => Err("Expected Int".to_string()),
    }
}

fn extract_u32(v: &Value) -> Result<u32, String> {
    match v {
        Value::Int(i) => Ok(*i as u32),
        _ => Err("Expected Int".to_string()),
    }
}

fn extract_bool(v: &Value) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err("Expected Bool".to_string()),
    }
}

fn extract_bytes(v: &Value) -> Result<Vec<u8>, String> {
    match v {
        Value::Bytes(b) => Ok(b.clone()),
        Value::String(s) => Ok(s.as_bytes().to_vec()),
        _ => Err("Expected Bytes or String".to_string()),
    }
}

fn extract_string_list(v: &Value) -> Result<Vec<String>, String> {
    match v {
        Value::List(l) => {
            let list = l.borrow();
            let mut res = Vec::new();
            for item in list.iter() {
                res.push(extract_string(item)?);
            }
            Ok(res)
        },
        _ => Err("Expected List of Strings".to_string()),
    }
}
