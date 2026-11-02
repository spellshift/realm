use crate::ast::{Argument, Value};
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::eval::functions::{call_value, evaluate_arg};
use crate::interpreter::eval::utils::to_iterable;
use crate::token::Span;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

/// `map(function, iterable)`: Applies `function` to every item of `iterable` and returns a list of the results.
///
/// **Parameters**
/// - `function` (Callable): Function to apply to each element.
/// - `iterable` (Iterable): The iterable to transform.
///
/// **Returns**
/// - `list`: A list of transformed elements.
pub(crate) fn builtin_map(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 2 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "map() takes exactly 2 arguments",
            span,
        );
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;

    let items = to_iterable(interp, &iterable_val, span)?;
    let mut results = Vec::new();

    for item in items {
        let res = call_value(interp, &func_val, core::slice::from_ref(&item), span)?;
        results.push(res);
    }

    Ok(Value::List(Arc::new(RwLock::new(results))))
}
