use super::ArgCheck;
use crate::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;

pub fn decode_utf16le_bytes(b: &[u8]) -> Result<String, String> {
    if !b.len().is_multiple_of(2) {
        return Err(
            "UnicodeDecodeError: UTF-16LE input bytes length must be a multiple of 2".into(),
        );
    }

    let utf16_units: Vec<u16> = b
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    String::from_utf16(&utf16_units).map_err(|e| alloc::format!("UnicodeDecodeError: {}", e))
}

pub fn handle_bytes_methods(
    b: &[u8],
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "decode" => Some((|| {
            args.require_range(0, 1, "decode")?;
            let encoding = if args.is_empty() {
                "utf8".into()
            } else {
                match &args[0] {
                    Value::String(enc) => enc.to_lowercase().replace(['-', '_'], ""),
                    Value::Bytes(b) => core::str::from_utf8(b)
                        .map_err(|_| {
                            "TypeError: decode() argument 'encoding' must be str".to_string()
                        })?
                        .to_lowercase()
                        .replace(['-', '_'], ""),
                    _ => return Err("TypeError: decode() argument 'encoding' must be str".into()),
                }
            };

            match encoding.as_str() {
                "utf8" => match String::from_utf8(b.to_vec()) {
                    Ok(s) => Ok(Value::String(s)),
                    Err(e) => Err(alloc::format!("UnicodeDecodeError: {}", e)),
                },
                "utf16le" => decode_utf16le_bytes(b).map(Value::String),
                _ => Err(alloc::format!("LookupError: unknown encoding: {}", args[0])),
            }
        })()),
        _ => None,
    }
}
