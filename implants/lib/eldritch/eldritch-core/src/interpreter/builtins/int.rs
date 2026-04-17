use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use spin::RwLock;

/// `int(x)`: Converts a number or string to an integer.
///
/// If x is a number, return x.__int__(). For floating point numbers, this truncates towards zero.
/// If x is not a number or if base is given, then x must be a string, bytes, or bytearray instance representing an integer literal in the given base.
pub fn builtin_int(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, NativeError> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    if args.len() > 2 {
        return Err(NativeError::runtime_error(format!(
            "int() takes at most 2 arguments ({} given)",
            args.len()
        )));
    }

    let x = &args[0];
    let base = if args.len() == 2 {
        match &args[1] {
            Value::Int(i) => Some(*i),
            _ => return Err(NativeError::runtime_error("int() base must be an integer")),
        }
    } else {
        None
    };

    if let Some(base) = base {
        // Explicit base provided
        match x {
            Value::String(s) => parse_int_string(s, base),
            _ => Err(NativeError::runtime_error(
                "int() can't convert non-string with explicit base",
            )),
        }
    } else {
        // No base provided
        match x {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Float(f) => Ok(Value::Int(*f as i64)), // Truncate
            Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
            Value::String(s) => parse_int_string(s, 10),
            _ => Err(NativeError::runtime_error(format!(
                "int() argument must be a string, bytes or number, not '{}'",
                get_type_name(x)
            ))),
        }
    }
}

fn parse_int_string(s: &str, base: i64) -> Result<Value, NativeError> {
    if (base != 0 && base < 2) || base > 36 {
        return Err(NativeError::runtime_error(
            "int() base must be >= 2 and <= 36, or 0",
        ));
    }

    let trimmed = s.trim();
    let (is_neg, clean_s) = if trimmed.starts_with('-') {
        (true, &trimmed[1..])
    } else if trimmed.starts_with('+') {
        (false, &trimmed[1..])
    } else {
        (false, trimmed)
    };

    let radix = if base == 0 {
        if clean_s.starts_with("0x") || clean_s.starts_with("0X") {
            16
        } else if clean_s.starts_with("0o") || clean_s.starts_with("0O") {
            8
        } else if clean_s.starts_with("0b") || clean_s.starts_with("0B") {
            2
        } else {
            10
        }
    } else {
        base as u32
    };

    let clean_s_no_prefix =
        if radix == 16 && (clean_s.starts_with("0x") || clean_s.starts_with("0X")) {
            &clean_s[2..]
        } else if radix == 8 && (clean_s.starts_with("0o") || clean_s.starts_with("0O")) {
            &clean_s[2..]
        } else if radix == 2 && (clean_s.starts_with("0b") || clean_s.starts_with("0B")) {
            &clean_s[2..]
        } else {
            clean_s
        };

    let uval = u64::from_str_radix(clean_s_no_prefix, radix).map_err(|_| {
        if base == 0 || base == 10 {
            NativeError::runtime_error(format!(
                "invalid literal for int() with base {radix}: '{s}'"
            ))
        } else {
            NativeError::runtime_error(format!("invalid literal for int() with base {base}: '{s}'"))
        }
    })?;

    if is_neg {
        // For negative, absolute value can be at most i64::MAX + 1 (which is abs(i64::MIN))
        // i64::MIN is -9223372036854775808
        // i64::MAX is  9223372036854775807
        // abs(i64::MIN) is 9223372036854775808 (u64)
        if uval > i64::MAX as u64 + 1 {
            return Err(NativeError::runtime_error(format!(
                "int() literal too large: '{s}'" // Python behavior for overflow in fixed-width types? Python ints are arbitrary precision.
                                                 // But eldritch uses i64. We should error on overflow.
            )));
        }
        // Wrapping negation for i64::MIN case
        Ok(Value::Int((uval as i64).wrapping_neg()))
    } else {
        if uval > i64::MAX as u64 {
            return Err(NativeError::runtime_error(format!(
                "int() literal too large: '{s}'"
            )));
        }
        Ok(Value::Int(uval as i64))
    }
}
