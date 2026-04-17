use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

pub fn builtin_eprint(
    env: &Arc<RwLock<Environment>>,
    args: &[Value],
) -> Result<Value, NativeError> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }

    env.read().printer.print_err(&out);
    Ok(Value::None)
}
