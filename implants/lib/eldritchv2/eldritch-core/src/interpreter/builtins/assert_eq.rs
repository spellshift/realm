use crate::ast::{Environment, Value};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `assert_eq(a, b)`: Aborts if `a` is not equal to `b`.
///
/// **Parameters**
/// - `a` (Any): Left operand.
/// - `b` (Any): Right operand.
pub fn builtin_assert_eq(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "assert_eq() takes exactly two arguments ({} given)",
            args.len()
        ));
    }
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}
