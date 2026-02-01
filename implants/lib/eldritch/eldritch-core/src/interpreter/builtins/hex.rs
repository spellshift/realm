use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `hex(x)`: Return the hexadecimal representation of an integer.
///
/// **Parameters**
/// - `x` (Int): The integer to convert.
pub fn builtin_hex(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "hex() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::Int(i) => {
            if *i < 0 {
                Ok(Value::String(format!("-0x{:x}", i.unsigned_abs())))
            } else {
                Ok(Value::String(format!("0x{:x}", i)))
            }
        }
        _ => Err(format!(
            "hex() argument must be an integer, not '{}'",
            get_type_name(&args[0])
        )),
    }
}
