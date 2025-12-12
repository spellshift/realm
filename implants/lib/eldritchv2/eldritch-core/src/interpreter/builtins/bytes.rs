use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

pub fn builtin_bytes(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Bytes(Vec::new()));
    }
    if args.len() != 1 {
        return Err("bytes() expects exactly one argument".to_string());
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Bytes(s.as_bytes().to_vec())),
        Value::List(l) => {
            let list = l.read();
            let mut bytes = Vec::with_capacity(list.len());
            for item in list.iter() {
                match item {
                    Value::Int(i) => {
                        if *i < 0 || *i > 255 {
                            return Err(format!(
                                "bytes() list items must be integers in range 0-255, got {i}"
                            ));
                        }
                        bytes.push(*i as u8);
                    }
                    _ => {
                        return Err(format!(
                            "bytes() list items must be integers, got {}",
                            get_type_name(item)
                        ))
                    }
                }
            }
            Ok(Value::Bytes(bytes))
        }
        Value::Int(i) => {
            if *i < 0 {
                return Err("bytes() argument cannot be negative".to_string());
            }
            Ok(Value::Bytes(vec![0; *i as usize]))
        }
        _ => Err(format!(
            "bytes() argument must be a string, list of integers, or integer size, not '{}'",
            get_type_name(&args[0])
        )),
    }
}
