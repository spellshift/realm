use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

/// `range(stop)` or `range(start, stop[, step])`: Returns a sequence of numbers.
///
/// **Parameters**
/// - `start` (Int): The start value (inclusive). Defaults to 0.
/// - `stop` (Int): The stop value (exclusive).
/// - `step` (Int): The step size. Defaults to 1.
pub fn builtin_range(
    _env: &Arc<RwLock<Environment>>,
    args: &[Value],
) -> Result<Value, NativeError> {
    let (start, end, step) = match args {
        [Value::Int(end)] => (0, *end, 1),
        [Value::Int(start), Value::Int(end)] => (*start, *end, 1),
        [Value::Int(start), Value::Int(end), Value::Int(step)] => (*start, *end, *step),
        _ => {
            return Err(NativeError::type_error(
                "range expects 1-3 integer arguments",
            ));
        }
    };
    if step == 0 {
        return Err(NativeError::value_error("range() arg 3 must not be zero"));
    }

    let mut list = Vec::new();
    let mut curr = start;
    if step > 0 {
        while curr < end {
            list.push(Value::Int(curr));
            curr += step;
        }
    } else {
        while curr > end {
            list.push(Value::Int(curr));
            curr += step;
        }
    }
    Ok(Value::List(Arc::new(RwLock::new(list))))
}
