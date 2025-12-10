use crate::ast::{Environment, Value};
use crate::token::Span;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use spin::RwLock;

/// `print(*args)`: Prints objects to the standard output.
///
/// Converts each argument to a string and prints it to the standard output,
/// separated by spaces.
pub fn builtin_print(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }

    // TODO: Pass actual span
    env.read()
        .printer
        .print_out(&Span::new(0, 0, 0), &out);
    Ok(Value::None)
}
