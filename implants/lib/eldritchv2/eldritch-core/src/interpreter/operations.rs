use super::super::ast::{Environment, Expr, Value};
use super::super::token::{Span, TokenKind};
use super::core::Interpreter;
use super::error::{EldritchError, EldritchErrorKind};
use super::eval::evaluate;
use super::introspection::{get_type_name, is_truthy};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use core::cmp::Ordering;
use spin::RwLock;

#[cfg(feature = "std")]
extern crate std;

pub fn adjust_slice_indices(
    length: i64,
    start: &Option<i64>,
    stop: &Option<i64>,
    step: i64,
) -> (i64, i64) {
    let start_val = if let Some(s) = start {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s >= length {
                length - 1
            } else if s < 0 {
                -1
            } else {
                s
            }
        } else if s < 0 {
            0
        } else if s > length {
            length
        } else {
            s
        }
    } else if step < 0 {
        length - 1
    } else {
        0
    };

    let stop_val = if let Some(s) = stop {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s < -1 {
                -1
            } else if s >= length {
                length - 1
            } else {
                s
            }
        } else if s < 0 {
            0
        } else if s > length {
            length
        } else {
            s
        }
    } else if step < 0 {
        -1
    } else {
        length
    };

    (start_val, stop_val)
}

pub fn compare_values(a: &Value, b: &Value) -> Result<Ordering, String> {
    // This function is kept for backward compatibility or explicit usage,
    // but Value now implements Ord so we can just use a.cmp(b) if types match
    // or return error if types mismatch (Python-like behavior for < >).
    // The previous implementation enforced type matching.

    // We should maintain the behavior that mismatched types are not comparable
    // for < > except for numbers? Python 3 raises TypeError.
    // The Ord implementation on Value defines a total order for ALL types.
    // However, the runtime behavior for < > typically wants TypeError for incompatible types.
    // But `compare_values` was used by `sorted` and comparisons.

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

pub(crate) fn evaluate_comprehension_generic<F>(
    interp: &mut Interpreter,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
    mut insert_fn: F,
) -> Result<(), EldritchError>
where
    F: FnMut(&mut Interpreter) -> Result<(), EldritchError>,
{
    let iterable_val = evaluate(interp, iterable)?;
    let items = super::eval::to_iterable(interp, &iterable_val, iterable.span)?;

    let printer = interp.env.read().printer.clone();
    let comp_env = Arc::new(RwLock::new(Environment {
        parent: Some(interp.env.clone()),
        values: BTreeMap::new(),
        printer,
        libraries: BTreeSet::new(),
    }));
    let original_env = interp.env.clone();
    interp.env = comp_env;

    for item in items {
        interp.define_variable(var, item);
        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            insert_fn(interp)?;
        }
    }
    interp.env = original_env;
    Ok(())
}

pub(crate) fn apply_arithmetic_op(
    interp: &Interpreter,
    a: &Value,
    op: &TokenKind,
    b: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    match (a, op, b) {
        (Value::Int(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Int(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Int(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Int(a), TokenKind::Slash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float((*a as f64) / (*b as f64)))
        }

        (Value::Float(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(a), TokenKind::Slash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float(a / b))
        }

        // Mixed
        (Value::Int(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
        (Value::Int(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float((*a as f64) - b)),
        (Value::Int(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float((*a as f64) * b)),
        (Value::Int(a), TokenKind::Slash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float((*a as f64) / b))
        }

        (Value::Float(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Float(a + (*b as f64))),
        (Value::Float(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Float(a - (*b as f64))),
        (Value::Float(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Float(a * (*b as f64))),
        (Value::Float(a), TokenKind::Slash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float(a / (*b as f64)))
        }

        // Floor Div and Modulo
        (Value::Float(a), TokenKind::SlashSlash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / b)))
            }
        }
        (Value::Int(a), TokenKind::SlashSlash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((*a as f64).div_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(*a as f64 / b)))
            }
        }
        (Value::Float(a), TokenKind::SlashSlash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(*b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / *b as f64)))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Int(a), TokenKind::Percent, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((*a as f64).rem_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let a = *a as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(*b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                let b = *b as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }

        (Value::Int(a), TokenKind::SlashSlash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            let mut res = a / b;
            if (a % b != 0) && ((*a < 0) ^ (*b < 0)) {
                res -= 1;
            }
            Ok(Value::Int(res))
        }
        (Value::Int(a), TokenKind::Percent, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            let res = ((a % b) + b) % b;
            Ok(Value::Int(res))
        }

        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "unsupported operand type(s) for {}: '{}' and '{}'",
                match op {
                    TokenKind::Plus => "+",
                    TokenKind::Minus => "-",
                    TokenKind::Star => "*",
                    TokenKind::Slash => "/",
                    TokenKind::SlashSlash => "//",
                    TokenKind::Percent => "%",
                    _ => "?",
                },
                get_type_name(a),
                get_type_name(b)
            ),
            span,
        ),
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

pub(crate) fn apply_bitwise_op(
    interp: &Interpreter,
    a: &Value,
    op: &TokenKind,
    b: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => match op {
            TokenKind::BitAnd => Ok(Value::Int(a & b)),
            TokenKind::BitOr => Ok(Value::Int(a | b)),
            TokenKind::BitXor => Ok(Value::Int(a ^ b)),
            TokenKind::LShift => Ok(Value::Int(a << b)),
            TokenKind::RShift => Ok(Value::Int(a >> b)),
            _ => unreachable!(),
        },
        (Value::Set(a), Value::Set(b)) => match op {
            TokenKind::BitAnd => {
                #[allow(clippy::mutable_key_type)]
                let intersection: BTreeSet<Value> =
                    a.read().intersection(&b.read()).cloned().collect();
                Ok(Value::Set(Arc::new(RwLock::new(intersection))))
            }
            TokenKind::BitOr => {
                #[allow(clippy::mutable_key_type)]
                let union: BTreeSet<Value> = a.read().union(&b.read()).cloned().collect();
                Ok(Value::Set(Arc::new(RwLock::new(union))))
            }
            TokenKind::BitXor => {
                #[allow(clippy::mutable_key_type)]
                let symmetric_difference: BTreeSet<Value> =
                    a.read().symmetric_difference(&b.read()).cloned().collect();
                Ok(Value::Set(Arc::new(RwLock::new(symmetric_difference))))
            }
            // Note: Minus is not bitwise, handled in arithmetic/sets
            _ => interp.error(
                EldritchErrorKind::TypeError,
                "Invalid bitwise operator for sets",
                span,
            ),
        },
        (Value::Dictionary(a), Value::Dictionary(b)) if matches!(op, TokenKind::BitOr) => {
            // Dict union (merge)
            let mut new_dict = a.read().clone();
            for (k, v) in b.read().iter() {
                new_dict.insert(k.clone(), v.clone());
            }
            Ok(Value::Dictionary(Arc::new(RwLock::new(new_dict))))
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "unsupported operand type(s) for {}: '{}' and '{}'",
                match op {
                    TokenKind::BitAnd => "&",
                    TokenKind::BitOr => "|",
                    TokenKind::BitXor => "^",
                    TokenKind::LShift => "<<",
                    TokenKind::RShift => ">>",
                    _ => "?",
                },
                get_type_name(a),
                get_type_name(b)
            ),
            span,
        ),
    }
}
