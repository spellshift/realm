use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use core::char;
use spin::RwLock;

/// `chr(i)`: Return the string representing a character whose Unicode code point is the integer `i`.
///
/// **Parameters**
/// - `i` (Int): The integer code point.
pub fn builtin_chr(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "chr() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::Int(i) => {
            // Valid range for char is roughly 0 to 0x10FFFF
            // Rust char::from_u32 checks this.
            if *i < 0 || *i > 0x10FFFF {
                return Err(format!("chr() arg not in range(0x110000)"));
            }
            match char::from_u32(*i as u32) {
                Some(c) => Ok(Value::String(String::from(c))),
                None => Err(format!("chr() arg not in range(0x110000)")),
            }
        }
        _ => Err(format!(
            "TypeError: an integer is required (got type {})",
            get_type_name(&args[0])
        )),
    }
}
