use super::super::super::ast::Value;
use super::super::super::token::Span;
use super::super::core::Interpreter;
use super::super::error::{EldritchError, EldritchErrorKind};
use super::super::introspection::get_type_name;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

pub(crate) fn to_iterable(
    interp: &Interpreter,
    val: &Value,
    span: Span,
) -> Result<Vec<Value>, EldritchError> {
    match val {
        Value::List(l) => Ok(l.read().clone()),
        Value::Tuple(t) => Ok(t.clone()),
        Value::Set(s) => Ok(s.read().iter().cloned().collect()),
        Value::Dictionary(d) => Ok(d.read().keys().cloned().collect()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("Type '{:?}' is not iterable", get_type_name(val)),
            span,
        ),
    }
}
