use super::super::super::ast::{Argument, Expr, Value};
use super::super::super::token::Span;
use super::super::core::{Flow, Interpreter};
use super::super::error::{EldritchError, EldritchErrorKind};
use super::functions::{call_value, evaluate_arg};
use super::utils::to_iterable;
use super::{MAX_RECURSION_DEPTH, evaluate};
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

// Builtin helper for checking truthiness without full import
fn is_truthy(val: &Value) -> bool {
    super::super::super::interpreter::introspection::is_truthy(val)
}

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

pub(crate) fn builtin_filter(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 2 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "filter() takes exactly 2 arguments",
            span,
        );
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let items = to_iterable(interp, &iterable_val, span)?;

    let mut results = Vec::new();
    for item in items {
        let keep = if let Value::None = func_val {
            is_truthy(&item)
        } else {
            let res = call_value(interp, &func_val, core::slice::from_ref(&item), span)?;
            is_truthy(&res)
        };
        if keep {
            results.push(item);
        }
    }
    Ok(Value::List(Arc::new(RwLock::new(results))))
}

pub(crate) fn builtin_reduce(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() < 2 || args.len() > 3 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "reduce() takes 2 or 3 arguments",
            span,
        );
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let mut items = to_iterable(interp, &iterable_val, span)?.into_iter();

    let mut acc = if args.len() == 3 {
        evaluate_arg(interp, &args[2])?
    } else {
        match items.next() {
            Some(v) => v,
            None => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "reduce() of empty sequence with no initial value",
                    span,
                );
            }
        }
    };

    for item in items {
        acc = call_value(interp, &func_val, &[acc, item], span)?;
    }
    Ok(acc)
}

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

pub(crate) fn builtin_eval(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 1 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "eval() takes exactly 1 argument",
            span,
        );
    }

    let code_val = evaluate_arg(interp, &args[0])?;
    let code = match code_val {
        Value::String(s) => s,
        _ => {
            return interp.error(
                EldritchErrorKind::TypeError,
                "eval() argument must be a string",
                span,
            );
        }
    };

    if interp.depth >= MAX_RECURSION_DEPTH {
        return interp.error(
            EldritchErrorKind::RecursionError,
            "Recursion limit exceeded",
            span,
        );
    }

    // Create a new interpreter instance that shares the environment
    // We manually construct it to avoid re-loading builtins and to set depth
    let mut temp_interp = Interpreter {
        env: interp.env.clone(),
        flow: Flow::Next,
        depth: interp.depth + 1,
        call_stack: interp.call_stack.clone(),
        current_func_name: "<eval>".to_string(),
        is_scope_owner: false,
    };

    match temp_interp.interpret(&code) {
        Ok(v) => Ok(v),
        Err(e) => interp.error(EldritchErrorKind::RuntimeError, &e, span),
    }
}
