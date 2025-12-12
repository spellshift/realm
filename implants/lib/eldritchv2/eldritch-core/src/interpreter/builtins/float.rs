use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

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
        Value::String(s) => {
            let s_trimmed = s.trim();
            // Handle inf/nan
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
                        // Check if literal denoted value too large (if standard parser returns inf for large value)
                        // Prompt says: "The call fails if the literal denotes a value too large to represent as a finite float."
                        // Rust's parse returns inf for overflow.
                        // But for "inf" string it is valid.
                        // Distinguishing overflow from explicit "inf" is handled by above checks.
                        // So if we reach here and get infinity, it was overflow.
                        return Err(format!("float() literal too large: {s}"));
                    }
                    Ok(Value::Float(f))
                }
                Err(_) => Err(format!("could not convert string to float: '{s}'")),
            }
        }
        _ => Err(format!(
            "float() argument must be a string or a number, not '{}'",
            get_type_name(&args[0])
        )),
    }
}
