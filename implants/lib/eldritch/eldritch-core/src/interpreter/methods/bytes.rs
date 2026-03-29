use super::ArgCheck;
use crate::ast::Value;
use alloc::string::String;

pub fn handle_bytes_methods(
    b: &[u8],
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "decode" => Some((|| {
            args.require(0, "decode")?;
            match String::from_utf8(b.to_vec()) {
                Ok(s) => Ok(Value::String(s)),
                Err(e) => Err(alloc::format!("UnicodeDecodeError: {}", e)),
            }
        })()),
        _ => None,
    }
}
