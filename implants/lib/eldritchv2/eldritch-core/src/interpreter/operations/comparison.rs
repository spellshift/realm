use crate::ast::Value;
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::introspection::get_type_name;
use crate::token::{Span, TokenKind};
use alloc::format;
use alloc::string::String;
use core::cmp::Ordering;

pub(crate) fn values_equal(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => f1.is_nan() && f2.is_nan(),
        _ => false,
    }
}

pub fn compare_values(a: &Value, b: &Value) -> Result<Ordering, String> {
    match (a, b) {
        (Value::Int(i1), Value::Float(f2)) => Ok((*i1 as f64).total_cmp(f2)),
        (Value::Float(f1), Value::Int(i2)) => Ok(f1.total_cmp(&(*i2 as f64))),
        _ => {
            if core::mem::discriminant(a) == core::mem::discriminant(b) {
                Ok(a.cmp(b))
            } else {
                Err(format!(
                    "Type mismatch or unsortable types: {} <-> {}",
                    get_type_name(a),
                    get_type_name(b)
                ))
            }
        }
    }
}

pub(crate) fn apply_comparison_op(
    interp: &Interpreter,
    a: &Value,
    op: &TokenKind,
    b: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    // Explicitly handle mixed Int/Float equality
    if matches!(op, TokenKind::Eq) {
        match (a, b) {
            (Value::Int(i), Value::Float(f)) => return Ok(Value::Bool(*i as f64 == *f)),
            (Value::Float(f), Value::Int(i)) => return Ok(Value::Bool(*f == *i as f64)),
            _ => return Ok(Value::Bool(a == b)),
        }
    }
    if matches!(op, TokenKind::NotEq) {
        match (a, b) {
            (Value::Int(i), Value::Float(f)) => return Ok(Value::Bool(*i as f64 != *f)),
            (Value::Float(f), Value::Int(i)) => return Ok(Value::Bool(*f != *i as f64)),
            _ => return Ok(Value::Bool(a != b)),
        }
    }

    match op {
        TokenKind::Eq | TokenKind::NotEq => unreachable!(), // Handled above
        _ => {
            // For <, >, <=, >= we use Value::partial_cmp (which delegates to Ord)

            // Check numbers first
            match (a, b) {
                (Value::Int(i1), Value::Float(f2)) => {
                    let f1 = *i1 as f64;
                    match op {
                        TokenKind::Lt => Ok(Value::Bool(f1 < *f2)),
                        TokenKind::Gt => Ok(Value::Bool(f1 > *f2)),
                        TokenKind::LtEq => Ok(Value::Bool(f1 <= *f2)),
                        TokenKind::GtEq => Ok(Value::Bool(f1 >= *f2)),
                        _ => unreachable!(),
                    }
                }
                (Value::Float(f1), Value::Int(i2)) => {
                    let f2 = *i2 as f64;
                    match op {
                        TokenKind::Lt => Ok(Value::Bool(*f1 < f2)),
                        TokenKind::Gt => Ok(Value::Bool(*f1 > f2)),
                        TokenKind::LtEq => Ok(Value::Bool(*f1 <= f2)),
                        TokenKind::GtEq => Ok(Value::Bool(*f1 >= f2)),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    // Mismatched types
                    if core::mem::discriminant(a) != core::mem::discriminant(b) {
                        return interp.error(
                            EldritchErrorKind::TypeError,
                            &format!(
                                "'{}' not supported between instances of '{}' and '{}'",
                                match op {
                                    TokenKind::Lt => "<",
                                    TokenKind::Gt => ">",
                                    TokenKind::LtEq => "<=",
                                    TokenKind::GtEq => ">=",
                                    _ => unreachable!(),
                                },
                                get_type_name(a),
                                get_type_name(b)
                            ),
                            span,
                        );
                    }
                    // Same types, use Ord
                    match a.cmp(b) {
                        core::cmp::Ordering::Less => Ok(Value::Bool(matches!(
                            op,
                            TokenKind::Lt | TokenKind::LtEq | TokenKind::NotEq
                        ))),
                        core::cmp::Ordering::Equal => Ok(Value::Bool(matches!(
                            op,
                            TokenKind::LtEq | TokenKind::GtEq | TokenKind::Eq
                        ))),
                        core::cmp::Ordering::Greater => Ok(Value::Bool(matches!(
                            op,
                            TokenKind::Gt | TokenKind::GtEq | TokenKind::NotEq
                        ))),
                    }
                }
            }
        }
    }
}
