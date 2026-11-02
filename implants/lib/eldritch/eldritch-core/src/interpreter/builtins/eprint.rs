use crate::ast::{Environment, Value};
use crate::token::Span;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

/// `eprint(*args)`: Prints objects to the standard error stream.
///
/// Converts each argument to a string and prints it to `stderr`,
/// separated by spaces.
///
/// **Returns**
/// - `None`
pub fn builtin_eprint(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }

    // TODO: Pass actual span
    env.read().printer.print_err(&Span::new(0, 0, 0), &out);
    Ok(Value::None)
}
