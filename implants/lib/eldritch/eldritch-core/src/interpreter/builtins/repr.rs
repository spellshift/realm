use crate::ast::{Environment, Value};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

pub fn builtin_repr(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "repr() takes exactly one argument ({} given)",
            args.len()
        ));
    }

    // Default formatting uses Debug for now, or display for strings differently?
    // Value::Display formats strings as-is. Value::Debug formats them with quotes.
    // We want quotes.
    // Also "repr(x) formats its argument as a string. All strings in the result are double-quoted."
    // Value::Debug uses Rust's Debug which uses double quotes for strings.
    // For invalid UTF-8 (Bytes treated as string?), Eldritch strings are always valid UTF-8.
    // So normal Debug should suffice for valid strings.
    // For Bytes, Debug prints `[1, 2, ...]`.
    // If the user wants `b'...'` style for bytes, I might need custom logic.
    // But `Value::String` logic:

    match &args[0] {
        Value::String(s) => Ok(Value::String(format!("{s:?}"))),
        _ => Ok(Value::String(format!("{:?}", args[0]))),
    }
}
