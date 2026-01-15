use super::super::super::ast::{Expr, Value};
use super::super::super::token::Span;
use super::super::core::Interpreter;
use super::super::error::{EldritchError, EldritchErrorKind};
use super::super::introspection::get_type_name;
use super::super::operations::adjust_slice_indices;
use super::evaluate;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub(crate) fn evaluate_index(
    interp: &mut Interpreter,
    obj: &Expr,
    index: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;
    let idx_val = evaluate(interp, index)?;

    match obj_val {
        Value::List(l) => {
            let idx_int = match idx_val {
                Value::Int(i) => i,
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "list indices must be integers",
                        index.span,
                    );
                }
            };
            let list = l.read();
            let true_idx = if idx_int < 0 {
                list.len() as i64 + idx_int
            } else {
                idx_int
            };
            if true_idx < 0 || true_idx as usize >= list.len() {
                return interp.error(
                    EldritchErrorKind::IndexError,
                    "List index out of range",
                    span,
                );
            }
            Ok(list[true_idx as usize].clone())
        }
        Value::Tuple(t) => {
            let idx_int = match idx_val {
                Value::Int(i) => i,
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "tuple indices must be integers",
                        index.span,
                    );
                }
            };
            let true_idx = if idx_int < 0 {
                t.len() as i64 + idx_int
            } else {
                idx_int
            };
            if true_idx < 0 || true_idx as usize >= t.len() {
                return interp.error(
                    EldritchErrorKind::IndexError,
                    "Tuple index out of range",
                    span,
                );
            }
            Ok(t[true_idx as usize].clone())
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            match dict.get(&idx_val) {
                Some(v) => Ok(v.clone()),
                None => interp.error(
                    EldritchErrorKind::KeyError,
                    &format!("KeyError: '{idx_val}'"),
                    span,
                ),
            }
        }
        Value::String(s) => {
            let idx_int = match idx_val {
                Value::Int(i) => i,
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "string indices must be integers",
                        index.span,
                    );
                }
            };
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let true_idx = if idx_int < 0 { len + idx_int } else { idx_int };
            if true_idx < 0 || true_idx as usize >= chars.len() {
                return interp.error(
                    EldritchErrorKind::IndexError,
                    "String index out of range",
                    span,
                );
            }
            Ok(Value::String(chars[true_idx as usize].to_string()))
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("'{}' object is not subscriptable", get_type_name(&obj_val)),
            obj.span,
        ),
    }
}

pub(crate) fn evaluate_slice(
    interp: &mut Interpreter,
    obj: &Expr,
    start: &Option<Box<Expr>>,
    stop: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
    span: Span,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;

    let step_val = if let Some(s) = step {
        match evaluate(interp, s)? {
            Value::Int(i) => i,
            _ => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "slice step must be an integer",
                    s.span,
                );
            }
        }
    } else {
        1
    };

    if step_val == 0 {
        return interp.error(
            EldritchErrorKind::ValueError,
            "slice step cannot be zero",
            span,
        );
    }

    let start_val_opt = if let Some(s) = start {
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "slice start must be an integer",
                    s.span,
                );
            }
        }
    } else {
        None
    };

    let stop_val_opt = if let Some(s) = stop {
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "slice stop must be an integer",
                    s.span,
                );
            }
        }
    } else {
        None
    };

    match obj_val {
        Value::List(l) => {
            let list = l.read();
            let len = list.len() as i64;
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);

            let mut result = Vec::new();
            let mut curr = i;

            if step_val > 0 {
                while curr < j {
                    if curr >= 0 && curr < len {
                        result.push(list[curr as usize].clone());
                    }
                    curr += step_val;
                }
            } else {
                while curr > j {
                    if curr >= 0 && curr < len {
                        result.push(list[curr as usize].clone());
                    }
                    curr += step_val;
                }
            }
            Ok(Value::List(Arc::new(RwLock::new(result))))
        }
        Value::Tuple(t) => {
            let len = t.len() as i64;
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
            let mut result = Vec::new();
            let mut curr = i;
            if step_val > 0 {
                while curr < j {
                    if curr >= 0 && curr < len {
                        result.push(t[curr as usize].clone());
                    }
                    curr += step_val;
                }
            } else {
                while curr > j {
                    if curr >= 0 && curr < len {
                        result.push(t[curr as usize].clone());
                    }
                    curr += step_val;
                }
            }
            Ok(Value::Tuple(result))
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
            let mut result_chars = Vec::new();
            let mut curr = i;
            if step_val > 0 {
                while curr < j {
                    if curr >= 0 && curr < len {
                        result_chars.push(chars[curr as usize]);
                    }
                    curr += step_val;
                }
            } else {
                while curr > j {
                    if curr >= 0 && curr < len {
                        result_chars.push(chars[curr as usize]);
                    }
                    curr += step_val;
                }
            }
            Ok(Value::String(result_chars.into_iter().collect()))
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("'{}' object is not subscriptable", get_type_name(&obj_val)),
            obj.span,
        ),
    }
}

pub(crate) fn evaluate_getattr(
    interp: &mut Interpreter,
    obj: &Expr,
    name: String,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;

    // Support dot access for dictionary keys (useful for modules)
    if let Value::Dictionary(d) = &obj_val {
        #[allow(clippy::collapsible_if)]
        if let Some(val) = d.read().get(&Value::String(name.clone())) {
            return Ok(val.clone());
        }
    }

    // Support Foreign Objects
    if let Value::Foreign(f) = &obj_val {
        if let Some(val) = f.get_attr(&name) {
            return Ok(val);
        }
        // Return a bound method where the receiver is the foreign object
        return Ok(Value::BoundMethod(Box::new(obj_val), name));
    }

    Ok(Value::BoundMethod(Box::new(obj_val), name))
}
