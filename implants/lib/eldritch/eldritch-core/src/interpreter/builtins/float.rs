use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `float(x=0.0)`: Converts a number or string to a floating point number.
///
/// **Parameters**
/// - `x` (Int | Float | String): The value to convert.
pub fn builtin_float(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Float(0.0));
    }
    if args.len() != 1 {
        return Err(format!(
            "float() takes at most 1 argument ({} given)",
            args.len()
        ));
    }

    match &args[0] {
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Value::String(s) => parse_float_str(s),
        Value::Bytes(b) => {
            let s = core::str::from_utf8(b)
                .map_err(|e| format!("could not convert bytes to float: {e}"))?;
            parse_float_str(s)
        }
        _ => Err(format!(
            "float() argument must be a string or a number, not '{}'",
            get_type_name(&args[0])
        )),
    }
}

fn parse_float_str(s: &str) -> Result<Value, String> {
    let s_trimmed = s.trim();
    let lower = s_trimmed.to_lowercase();
    if lower == "inf" || lower == "infinity" || lower == "+inf" || lower == "+infinity" {
        return Ok(Value::Float(f64::INFINITY));
    }
    if lower == "-inf" || lower == "-infinity" {
        return Ok(Value::Float(f64::NEG_INFINITY));
    }
    if lower == "nan" || lower == "+nan" || lower == "-nan" {
        return Ok(Value::Float(f64::NAN));
    }

    match s_trimmed.parse::<f64>() {
        Ok(f) => {
            if f.is_infinite() {
                return Err(format!("float() literal too large: {s}"));
            }
            Ok(Value::Float(f))
        }
        Err(_) => Err(format!("could not convert string to float: '{s}'")),
    }
}
