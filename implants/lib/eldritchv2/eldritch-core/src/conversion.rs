use super::ast::Value;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub trait FromValue: Sized {
    fn from_value(v: &Value) -> Result<Self, String>;
}

pub trait ToValue {
    fn to_value(self) -> Value;
}

// Implementations for FromValue
impl FromValue for i64 {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Int(i) => Ok(*i),
            _ => Err(format!("Expected Int, got {}", get_type_name(v))),
        }
    }
}

impl FromValue for f64 {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            _ => Err(format!("Expected Float or Int, got {}", get_type_name(v))),
        }
    }
}

impl FromValue for String {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Expected String, got {}", get_type_name(v))),
        }
    }
}

impl FromValue for bool {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Bool(b) => Ok(*b),
            _ => Err(format!("Expected Bool, got {}", get_type_name(v))),
        }
    }
}

impl FromValue for Vec<u8> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Bytes(b) => Ok(b.clone()),
            _ => Err(format!("Expected Bytes, got {}", get_type_name(v))),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::List(l) => {
                let list = l.read();
                let mut res = Vec::with_capacity(list.len());
                for item in list.iter() {
                    res.push(T::from_value(item)?);
                }
                Ok(res)
            }
            Value::Tuple(t) => {
                let mut res = Vec::with_capacity(t.len());
                for item in t.iter() {
                    res.push(T::from_value(item)?);
                }
                Ok(res)
            }
            // Should we support Set -> Vec conversion automatically?
            // Python typing sometimes allows Iterable[T].
            // But strict Vec<T> mapping usually implies order.
            // Let's stick to List/Tuple for now.
            _ => Err(format!("Expected List or Tuple, got {}", get_type_name(v))),
        }
    }
}

impl<K: FromValue + Ord, V: FromValue> FromValue for BTreeMap<K, V> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.read();
                let mut res = BTreeMap::new();
                for (key_val, val) in dict.iter() {
                    let k = K::from_value(key_val)?;
                    let v = V::from_value(val)?;
                    res.insert(k, v);
                }
                Ok(res)
            }
            _ => Err(format!("Expected Dictionary, got {}", get_type_name(v))),
        }
    }
}

impl FromValue for Value {
    fn from_value(v: &Value) -> Result<Self, String> {
        Ok(v.clone())
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::None => Ok(None),
            _ => Ok(Some(T::from_value(v)?)),
        }
    }
}

// Implementations for ToValue
impl ToValue for i64 {
    fn to_value(self) -> Value {
        Value::Int(self)
    }
}

impl ToValue for f64 {
    fn to_value(self) -> Value {
        Value::Float(self)
    }
}

impl ToValue for String {
    fn to_value(self) -> Value {
        Value::String(self)
    }
}

impl ToValue for () {
    fn to_value(self) -> Value {
        Value::None
    }
}

impl ToValue for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }
}

impl ToValue for Vec<u8> {
    fn to_value(self) -> Value {
        Value::Bytes(self)
    }
}

impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(self) -> Value {
        let list: Vec<Value> = self.into_iter().map(|i| i.to_value()).collect();
        Value::List(Arc::new(RwLock::new(list)))
    }
}

impl<K: ToValue + Ord, V: ToValue> ToValue for BTreeMap<K, V> {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        for (k, v) in self {
            map.insert(k.to_value(), v.to_value());
        }
        Value::Dictionary(Arc::new(RwLock::new(map)))
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(self) -> Value {
        match self {
            Some(v) => v.to_value(),
            None => Value::None,
        }
    }
}

impl ToValue for Value {
    fn to_value(self) -> Value {
        self
    }
}

// Trait for handling return types
pub trait IntoEldritchResult {
    fn into_eldritch_result(self) -> Result<Value, String>;
}

impl<T, E> IntoEldritchResult for Result<T, E>
where
    T: ToValue,
    E: ToString,
{
    fn into_eldritch_result(self) -> Result<Value, String> {
        self.map(|v| v.to_value()).map_err(|e| e.to_string())
    }
}

// Helper to get type name (duplicate from utils but avoids public exposure of utils)
fn get_type_name(v: &Value) -> &'static str {
    match v {
        Value::None => "NoneType",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::String(_) => "str",
        Value::Bytes(_) => "bytes",
        Value::List(_) => "list",
        Value::Tuple(_) => "tuple",
        Value::Dictionary(_) => "dict",
        Value::Set(_) => "set",
        Value::Function(_) => "function",
        Value::NativeFunction(_, _) => "native_function",
        Value::NativeFunctionWithKwargs(_, _) => "native_function_kwargs",
        Value::BoundMethod(_, _) => "bound_method",
        Value::Foreign(_) => "foreign_object",
    }
}

