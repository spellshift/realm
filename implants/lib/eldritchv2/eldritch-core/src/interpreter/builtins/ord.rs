use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `ord(c)`: Return the integer that represents the Unicode code point of the character `c`.
///
/// **Parameters**
/// - `c` (String | Bytes): A string of length 1 or bytes of length 1.
pub fn builtin_ord(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "ord() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::String(s) => {
            let mut chars = s.chars();
            if let Some(c) = chars.next()
                && chars.next().is_none()
            {
                return Ok(Value::Int(c as i64));
            }
            Err(format!(
                "ord() expected string of length 1, but string '{}' found",
                s
            ))
        }
        Value::Bytes(b) => {
            if b.len() == 1 {
                Ok(Value::Int(b[0] as i64))
            } else {
                Err(format!(
                    "ord() expected bytes of length 1, but bytes of length {} found",
                    b.len()
                ))
            }
        }
        _ => Err(format!(
            "TypeError: ord() expected string of length 1, but {} found",
            get_type_name(&args[0])
        )),
    }
}
