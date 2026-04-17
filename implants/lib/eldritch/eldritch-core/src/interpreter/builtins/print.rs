use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

/// `print(*args)`: Prints objects to the standard output.
///
/// Converts each argument to a string and prints it to the standard output,
/// separated by spaces.
pub fn builtin_print(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, NativeError> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }

    env.read().printer.print_out(&out);
    Ok(Value::None)
}
