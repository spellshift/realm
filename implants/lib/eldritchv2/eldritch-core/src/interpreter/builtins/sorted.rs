use crate::ast::{Argument, Expr, Value};
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::eval::evaluate;
use crate::interpreter::eval::functions::call_value;
use crate::interpreter::eval::utils::to_iterable;
use crate::interpreter::introspection::is_truthy;
use crate::token::Span;
use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub(crate) fn builtin_sorted(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    let mut iterable_arg: Option<&Expr> = None;
    let mut key_arg: Option<&Expr> = None;
    let mut reverse_arg: Option<&Expr> = None;

    for arg in args {
        match arg {
            Argument::Positional(e) => {
                if iterable_arg.is_none() {
                    iterable_arg = Some(e);
                } else {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "sorted() takes only 1 positional argument",
                        span,
                    );
                }
            }
            Argument::Keyword(name, e) => match name.as_str() {
                "key" => key_arg = Some(e),
                "reverse" => reverse_arg = Some(e),
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &format!("sorted() got an unexpected keyword argument '{name}'"),
                        span,
                    );
                }
            },
            _ => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "sorted() does not support *args or **kwargs unpacking",
                    span,
                );
            }
        }
    }

    let iterable_expr = iterable_arg.ok_or_else(|| {
        interp
            .error::<()>(
                EldritchErrorKind::TypeError,
                "sorted() missing 1 required positional argument: 'iterable'",
                span,
            )
            .unwrap_err()
    })?;

    let iterable_val = evaluate(interp, iterable_expr)?;
    let mut items = to_iterable(interp, &iterable_val, span)?;

    // Handle key
    if let Some(key_expr) = key_arg {
        let key_func = evaluate(interp, key_expr)?;
        if matches!(key_func, Value::None) {
            // sort normally
            items.sort();
        } else {
            // Decorated sort
            let mut decorated = Vec::with_capacity(items.len());
            for item in items.iter() {
                let k = call_value(interp, &key_func, core::slice::from_ref(item), span)?;
                decorated.push((k, item.clone()));
            }
            // Sort decorated
            // Value implements Ord
            decorated.sort_by(|a, b| a.0.cmp(&b.0));

            // Undecorate
            items = decorated.into_iter().map(|(_, v)| v).collect();
        }
    } else {
        items.sort();
    }

    // Handle reverse
    if let Some(rev_expr) = reverse_arg {
        let rev_val = evaluate(interp, rev_expr)?;
        if is_truthy(&rev_val) {
            items.reverse();
        }
    }

    Ok(Value::List(Arc::new(RwLock::new(items))))
}
