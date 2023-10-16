use anyhow::{anyhow, Result};
use base64::{Engine, engine::general_purpose};

pub fn encode_b64(content: String, encode_type: String) -> Result<String> {
    let encode_type = match encode_type.as_str() {
        "STANDARD" => {general_purpose::STANDARD},
        "STANDARD_NO_PAD" => {general_purpose::STANDARD_NO_PAD},
        "URL_SAFE" => {general_purpose::URL_SAFE},
        "URL_SAFE_NO_PAD" => {general_purpose::URL_SAFE_NO_PAD},
        _ => return Err(anyhow!("Invalid encode type. Valid types are: STANDARD, STANDARD_NO_PAD, URL_SAFE_PAD, URL_SAFE_NO_PAD"))
    };
    Ok(encode_type.encode(content.as_bytes()))
}
