use super::super::super::ast::{Expr, Value};
use super::super::super::token::{Span, TokenKind};
use super::super::core::Interpreter;
use super::super::error::{EldritchError, EldritchErrorKind, runtime_error};
use super::super::introspection::{get_type_name, is_truthy};
use super::super::operations::{
    apply_arithmetic_op, apply_bitwise_op, apply_comparison_op, compare_values, values_equal,
};
use super::evaluate;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Write;
use spin::RwLock;

pub(crate) fn apply_unary_op(
    interp: &mut Interpreter,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let val = evaluate(interp, right)?;
    match op {
        TokenKind::Minus => match val {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => interp.error(
                EldritchErrorKind::TypeError,
                "Unary '-' only valid for numbers",
                span,
            ),
        },
        TokenKind::Not => Ok(Value::Bool(!is_truthy(&val))),
        TokenKind::BitNot => match val {
            Value::Int(i) => Ok(Value::Int(!i)),
            _ => interp.error(
                EldritchErrorKind::TypeError,
                "Bitwise '~' only valid for integers",
                span,
            ),
        },
        _ => interp.error(
            EldritchErrorKind::SyntaxError,
            "Invalid unary operator",
            span,
        ),
    }
}

pub(crate) fn apply_logical_op(
    interp: &mut Interpreter,
    left: &Expr,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let left_val = evaluate(interp, left)?;
    match op {
        TokenKind::Or => {
            if is_truthy(&left_val) {
                return Ok(left_val);
            }
            evaluate(interp, right)
        }
        TokenKind::And => {
            if !is_truthy(&left_val) {
                return Ok(left_val);
            }
            evaluate(interp, right)
        }
        _ => interp.error(
            EldritchErrorKind::SyntaxError,
            "Invalid logical operator",
            span,
        ),
    }
}

fn evaluate_in(
    interp: &mut Interpreter,
    item: &Value,
    collection: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    match collection {
        Value::List(l) => {
            let list = l.read();
            Ok(Value::Bool(list.iter().any(|x| values_equal(x, item))))
        }
        Value::Tuple(t) => Ok(Value::Bool(t.iter().any(|x| values_equal(x, item)))),
        Value::Dictionary(d) => {
            let dict = d.read();
            // Check keys
            Ok(Value::Bool(dict.contains_key(item)))
        }
        Value::Set(s) => {
            let set = s.read();
            Ok(Value::Bool(set.contains(item)))
        }
        Value::String(s) => {
            let sub = match item {
                Value::String(ss) => ss,
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "'in <string>' requires string as left operand",
                        span,
                    );
                }
            };
            Ok(Value::Bool(s.contains(sub)))
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "argument of type '{}' is not iterable",
                get_type_name(collection)
            ),
            span,
        ),
    }
}

pub(crate) fn apply_binary_op(
    interp: &mut Interpreter,
    left: &Expr,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let a = evaluate(interp, left)?;
    let b = evaluate(interp, right)?;

    // Handle operations that are fully delegated
    if matches!(
        op,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::SlashSlash
            | TokenKind::Percent
    ) {
        // Some specific type combinations need special handling, but generic arithmetic is in helper.
        // Wait, string concatenation is + but logic is in apply_binary_op?
        // Let's check which are generic.

        // Only numbers are fully handled in apply_arithmetic_op
        if (matches!(a, Value::Int(_) | Value::Float(_))
            && matches!(b, Value::Int(_) | Value::Float(_)))
        {
            return apply_arithmetic_op(interp, &a, op, &b, span);
        }
    }

    // Comparisons
    if matches!(
        op,
        TokenKind::Eq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
    ) {
        // Sequence comparison is special (recursive).
        // Numbers and Mixed are handled in helper.
        match (&a, &b) {
            (Value::List(la), Value::List(lb)) => {
                let list_a = la.read();
                let list_b = lb.read();
                return compare_sequences(&list_a, &list_b, op.clone(), span);
            }
            (Value::Tuple(ta), Value::Tuple(tb)) => {
                return compare_sequences(ta, tb, op.clone(), span);
            }
            _ => return apply_comparison_op(interp, &a, op, &b, span),
        }
    }

    // Bitwise
    if matches!(
        op,
        TokenKind::BitAnd
            | TokenKind::BitOr
            | TokenKind::BitXor
            | TokenKind::LShift
            | TokenKind::RShift
    ) {
        return apply_bitwise_op(interp, &a, op, &b, span);
    }

    match (a, op.clone(), b) {
        // IN Operator
        (item, TokenKind::In, collection) => evaluate_in(interp, &item, &collection, span),
        (item, TokenKind::NotIn, collection) => {
            let res = evaluate_in(interp, &item, &collection, span)?;
            match res {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => unreachable!("evaluate_in always returns boolean or error"),
            }
        }

        // Non-numeric arithmetic (Sequences)

        // Set Difference (Minus)
        (Value::Set(a), TokenKind::Minus, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let difference: BTreeSet<Value> = a.read().difference(&b.read()).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(difference))))
        }

        (Value::String(a), TokenKind::Plus, Value::String(b)) => Ok(Value::String(a + &b)),
        (Value::String(a), TokenKind::Percent, b_val) => {
            // String formatting
            string_modulo_format(interp, &a, &b_val, span)
        }

        (Value::Bytes(a), TokenKind::Plus, Value::Bytes(b)) => {
            let mut new_bytes = a.clone();
            new_bytes.extend(b.iter());
            Ok(Value::Bytes(new_bytes))
        }

        (Value::Bytes(a), TokenKind::Star, Value::Int(n)) => {
            if n <= 0 {
                Ok(Value::Bytes(Vec::new()))
            } else {
                let mut new_bytes = Vec::with_capacity(a.len() * (n as usize));
                for _ in 0..n {
                    new_bytes.extend(a.iter());
                }
                Ok(Value::Bytes(new_bytes))
            }
        }
        (Value::Int(n), TokenKind::Star, Value::Bytes(a)) => {
            if n <= 0 {
                Ok(Value::Bytes(Vec::new()))
            } else {
                let mut new_bytes = Vec::with_capacity(a.len() * (n as usize));
                for _ in 0..n {
                    new_bytes.extend(a.iter());
                }
                Ok(Value::Bytes(new_bytes))
            }
        }

        // List concatenation (new list)
        (Value::List(mut a), TokenKind::Plus, Value::List(b)) => {
            // Optimization: If `a` is a temporary (unique), mutate in place
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                let list = rw_lock.get_mut();
                list.extend(b.read().clone());
                Ok(Value::List(a))
            } else {
                let mut new_list = a.read().clone();
                new_list.extend(b.read().clone());
                Ok(Value::List(Arc::new(RwLock::new(new_list))))
            }
        }

        // List repetition (Multiplication)
        (Value::List(a), TokenKind::Star, Value::Int(n)) => {
            let mut new_list = Vec::new();
            if n > 0 {
                let list_ref = a.read();
                for _ in 0..n {
                    new_list.extend(list_ref.clone());
                }
            }
            Ok(Value::List(Arc::new(RwLock::new(new_list))))
        }
        (Value::Int(n), TokenKind::Star, Value::List(a)) => {
            let mut new_list = Vec::new();
            if n > 0 {
                let list_ref = a.read();
                for _ in 0..n {
                    new_list.extend(list_ref.clone());
                }
            }
            Ok(Value::List(Arc::new(RwLock::new(new_list))))
        }

        // Tuple concatenation (new tuple)
        (Value::Tuple(a), TokenKind::Plus, Value::Tuple(b)) => {
            let mut new_tuple = a.clone();
            new_tuple.extend(b.clone());
            Ok(Value::Tuple(new_tuple))
        }

        // Tuple repetition
        (Value::Tuple(a), TokenKind::Star, Value::Int(n)) => {
            let mut new_tuple = Vec::new();
            if n > 0 {
                for _ in 0..n {
                    new_tuple.extend(a.clone());
                }
            }
            Ok(Value::Tuple(new_tuple))
        }
        (Value::Int(n), TokenKind::Star, Value::Tuple(a)) => {
            let mut new_tuple = Vec::new();
            if n > 0 {
                for _ in 0..n {
                    new_tuple.extend(a.clone());
                }
            }
            Ok(Value::Tuple(new_tuple))
        }

        // String repetition
        (Value::String(s), TokenKind::Star, Value::Int(n)) => {
            if n <= 0 {
                Ok(Value::String(String::new()))
            } else {
                Ok(Value::String(s.repeat(n as usize)))
            }
        }
        (Value::Int(n), TokenKind::Star, Value::String(s)) => {
            if n <= 0 {
                Ok(Value::String(String::new()))
            } else {
                Ok(Value::String(s.repeat(n as usize)))
            }
        }

        // Dict merge (new dict)
        (Value::Dictionary(mut a), TokenKind::Plus, Value::Dictionary(b)) => {
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                let dict = rw_lock.get_mut();
                for (k, v) in b.read().iter() {
                    dict.insert(k.clone(), v.clone());
                }
                Ok(Value::Dictionary(a))
            } else {
                let mut new_dict = a.read().clone();
                for (k, v) in b.read().iter() {
                    new_dict.insert(k.clone(), v.clone());
                }
                Ok(Value::Dictionary(Arc::new(RwLock::new(new_dict))))
            }
        }

        // Set union (new set) - Plus is deprecated for sets in favor of |
        (Value::Set(mut a), TokenKind::Plus, Value::Set(b)) => {
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                #[allow(clippy::mutable_key_type)]
                let set = rw_lock.get_mut();
                for item in b.read().iter() {
                    set.insert(item.clone());
                }
                Ok(Value::Set(a))
            } else {
                #[allow(clippy::mutable_key_type)]
                let mut new_set = a.read().clone();
                for item in b.read().iter() {
                    new_set.insert(item.clone());
                }
                Ok(Value::Set(Arc::new(RwLock::new(new_set))))
            }
        }

        (val_a, op_kind, val_b) => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "unsupported operand type(s) for {}: '{}' and '{}'",
                match op_kind {
                    TokenKind::Plus => "+",
                    TokenKind::Minus => "-",
                    TokenKind::Star => "*",
                    TokenKind::Slash => "/",
                    TokenKind::SlashSlash => "//",
                    TokenKind::Percent => "%",
                    TokenKind::BitAnd => "&",
                    TokenKind::BitOr => "|",
                    TokenKind::BitXor => "^",
                    TokenKind::LShift => "<<",
                    TokenKind::RShift => ">>",
                    _ => "binary op",
                },
                get_type_name(&val_a),
                get_type_name(&val_b)
            ),
            span,
        ),
    }
}

fn string_modulo_format(
    interp: &mut Interpreter,
    fmt_str: &str,
    val: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    let mut result = String::new();
    let mut chars = fmt_str.chars().peekable();
    let mut val_idx = 0;
    let vals: Vec<Value> = match val {
        Value::Tuple(t) => t.clone(),
        _ => vec![val.clone()],
    };

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                // If it is '%%', handle immediately
                if next == '%' {
                    chars.next();
                    result.push('%');
                    continue;
                }

                chars.next(); // Consume specifier

                if val_idx >= vals.len() {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "not enough arguments for format string",
                        span,
                    );
                }
                let v = &vals[val_idx];
                val_idx += 1;

                match next {
                    's' => {
                        result.push_str(&v.to_string());
                    }
                    'd' | 'i' | 'u' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val}");
                    }
                    'o' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:o}");
                    }
                    'x' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:x}");
                    }
                    'X' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:X}");
                    }
                    'e' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:e}");
                    }
                    'E' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:E}");
                    }
                    'f' | 'F' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:.6}",);
                    }
                    'g' | 'G' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:?}");
                    }
                    'r' => match v {
                        Value::String(s) => {
                            let _ = write!(result, "\"{s}\"");
                        }
                        _ => result.push_str(&v.to_string()),
                    },
                    _ => {
                        return interp.error(
                            EldritchErrorKind::ValueError,
                            &format!(
                                "unsupported format character '{}' (0x{:x})",
                                next, next as u32
                            ),
                            span,
                        );
                    }
                }
            } else {
                return interp.error(EldritchErrorKind::ValueError, "incomplete format key", span);
            }
        } else {
            result.push(c);
        }
    }

    if val_idx < vals.len() {
        return interp.error(
            EldritchErrorKind::TypeError,
            "not all arguments converted during string formatting",
            span,
        );
    }

    Ok(Value::String(result))
}

fn to_int_or_error(
    interp: &Interpreter,
    v: &Value,
    spec: char,
    span: Span,
) -> Result<i64, EldritchError> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Float(f) => Ok(*f as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "%{} format: a number is required, not {}",
                spec,
                get_type_name(v)
            ),
            span,
        ),
    }
}

fn to_float_or_error(
    interp: &Interpreter,
    v: &Value,
    spec: char,
    span: Span,
) -> Result<f64, EldritchError> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "%{} format: a number is required, not {}",
                spec,
                get_type_name(v)
            ),
            span,
        ),
    }
}

fn compare_sequences(
    seq_a: &[Value],
    seq_b: &[Value],
    op: TokenKind,
    span: Span,
) -> Result<Value, EldritchError> {
    // Lexicographical comparison
    let len_a = seq_a.len();
    let len_b = seq_b.len();
    let len = len_a.min(len_b);

    for i in 0..len {
        let val_a = &seq_a[i];
        let val_b = &seq_b[i];

        if val_a != val_b {
            return match op {
                TokenKind::Eq => Ok(Value::Bool(false)),
                TokenKind::NotEq => Ok(Value::Bool(true)),
                TokenKind::Lt => {
                    // Check if a < b
                    Ok(Value::Bool(
                        compare_values(val_a, val_b).is_ok_and(|ord| matches!(ord, Ordering::Less)),
                    ))
                }
                TokenKind::Gt => Ok(Value::Bool(
                    compare_values(val_a, val_b).is_ok_and(|ord| matches!(ord, Ordering::Greater)),
                )),
                TokenKind::LtEq => {
                    Ok(Value::Bool(compare_values(val_a, val_b).is_ok_and(|ord| {
                        matches!(ord, Ordering::Less | Ordering::Equal)
                    })))
                }
                TokenKind::GtEq => {
                    Ok(Value::Bool(compare_values(val_a, val_b).is_ok_and(|ord| {
                        matches!(ord, Ordering::Greater | Ordering::Equal)
                    })))
                }
                _ => runtime_error(span, "Invalid comparison operator for sequences"),
            };
        }
    }

    // If prefix matches, compare lengths
    match op {
        TokenKind::Eq => Ok(Value::Bool(len_a == len_b)),
        TokenKind::NotEq => Ok(Value::Bool(len_a != len_b)),
        TokenKind::Lt => Ok(Value::Bool(len_a < len_b)),
        TokenKind::Gt => Ok(Value::Bool(len_a > len_b)),
        TokenKind::LtEq => Ok(Value::Bool(len_a <= len_b)),
        TokenKind::GtEq => Ok(Value::Bool(len_a >= len_b)),
        _ => runtime_error(span, "Invalid comparison operator for sequences"),
    }
}
