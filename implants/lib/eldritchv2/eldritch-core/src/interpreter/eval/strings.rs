use super::super::super::ast::{FStringSegment, Value};
use super::super::core::Interpreter;
use super::super::error::EldritchError;
use super::evaluate;
use alloc::string::ToString;
use alloc::vec::Vec;

pub(crate) fn evaluate_fstring(
    interp: &mut Interpreter,
    segments: &[FStringSegment],
) -> Result<Value, EldritchError> {
    let mut parts = Vec::new();
    for segment in segments {
        match segment {
            FStringSegment::Literal(s) => parts.push(s.clone()),
            FStringSegment::Expression(expr) => {
                let val = evaluate(interp, expr)?;
                parts.push(val.to_string());
            }
        }
    }
    Ok(Value::String(parts.join("")))
}
