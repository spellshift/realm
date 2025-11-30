use crate::ast::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::cell::RefCell;
use alloc::rc::Rc;

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
                let list = l.borrow();
                let mut res = Vec::with_capacity(list.len());
                for item in list.iter() {
                    res.push(T::from_value(item)?);
                }
                Ok(res)
            },
            Value::Tuple(t) => {
                let mut res = Vec::with_capacity(t.len());
                for item in t.iter() {
                    res.push(T::from_value(item)?);
                }
                Ok(res)
            }
            _ => Err(format!("Expected List or Tuple, got {}", get_type_name(v))),
        }
    }
}

impl<K: FromValue + Ord, V: FromValue> FromValue for BTreeMap<K, V> {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Dictionary(d) => {
                let dict = d.borrow();
                let mut res = BTreeMap::new();
                for (key_str, val) in dict.iter() {
                    // Keys in Eldritch dicts are currently Strings.
                    // If K is not String, we might have issues if we strictly rely on K::from_value(Value::String).
                    // But let's assume K can parse from String Value.
                    let key_val = Value::String(key_str.clone());
                    let k = K::from_value(&key_val)?;
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

// Implementations for ToValue
impl ToValue for i64 {
    fn to_value(self) -> Value {
        Value::Int(self)
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
        Value::List(Rc::new(RefCell::new(list)))
    }
}

impl<K: ToValue + ToString, V: ToValue> ToValue for BTreeMap<K, V> {
    fn to_value(self) -> Value {
        let mut map = BTreeMap::new();
        for (k, v) in self {
            // Eldritch dict keys are strings. We use ToString.
            // Note: K: ToValue isn't strictly needed for the key if we just use ToString,
            // but for symmetry we might expect it. However, the internal BTreeMap is <String, Value>.
            map.insert(k.to_string(), v.to_value());
        }
        Value::Dictionary(Rc::new(RefCell::new(map)))
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

// Function trait and adapter
pub trait EldritchFunction<Marker> {
    fn call(&self, args: &[Value]) -> Result<Value, String>;
}

// Call stub helper
pub fn call_stub<Marker, F>(f: F, args: &[Value]) -> Result<Value, String>
where
    F: EldritchFunction<Marker>,
{
    f.call(args)
}

// Helper to get type name (duplicate from utils but avoids public exposure of utils)
fn get_type_name(v: &Value) -> &'static str {
    match v {
        Value::None => "NoneType",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::String(_) => "str",
        Value::Bytes(_) => "bytes",
        Value::List(_) => "list",
        Value::Tuple(_) => "tuple",
        Value::Dictionary(_) => "dict",
        Value::Function(_) => "function",
        Value::NativeFunction(_, _) => "native_function",
        Value::NativeFunctionWithKwargs(_, _) => "native_function_kwargs",
        Value::BoundMethod(_, _) => "bound_method",
        Value::Foreign(_) => "foreign_object",
    }
}

// Macro to implement EldritchFunction for tuples of arguments
macro_rules! impl_eldritch_fn {
    ($($arg:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        impl<Func, Ret, $($arg),*> EldritchFunction<($($arg,)*)> for Func
        where
            Func: Fn($($arg),*) -> Ret,
            Ret: IntoEldritchResult,
            $($arg: FromValue),*
        {
            fn call(&self, args: &[Value]) -> Result<Value, String> {
                // Count args
                let expected_len = 0 $( + { let _ = stringify!($arg); 1 } )*;
                if args.len() != expected_len {
                    return Err(format!("Expected {} arguments, got {}", expected_len, args.len()));
                }

                let mut args_iter = args.iter();
                // We use a closure to capture errors during extraction
                let res = self(
                    $(
                        match $arg::from_value(args_iter.next().unwrap()) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        },
                    )*
                );
                res.into_eldritch_result()
            }
        }
    }
}

impl_eldritch_fn!();
impl_eldritch_fn!(A);
impl_eldritch_fn!(A, B);
impl_eldritch_fn!(A, B, C);
impl_eldritch_fn!(A, B, C, D);
impl_eldritch_fn!(A, B, C, D, E);
impl_eldritch_fn!(A, B, C, D, E, F);
