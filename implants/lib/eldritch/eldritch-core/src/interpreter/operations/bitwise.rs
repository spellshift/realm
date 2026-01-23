use crate::ast::Value;
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::introspection::get_type_name;
use crate::token::{Span, TokenKind};
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::sync::Arc;
use spin::RwLock;

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
