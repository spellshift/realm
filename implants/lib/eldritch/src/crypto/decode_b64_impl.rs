use anyhow::{anyhow, Result};
use base64::{Engine, engine::general_purpose};

pub fn decode_b64(content: String, encode_type: String) -> Result<String> {
    let decode_type = match encode_type.as_str() {
        "STANDARD" => {general_purpose::STANDARD},
        "STANDARD_NO_PAD" => {general_purpose::STANDARD_NO_PAD},
        "URL_SAFE" => {general_purpose::URL_SAFE},
        "URL_SAFE_NO_PAD" => {general_purpose::URL_SAFE_NO_PAD},
        _ => return Err(anyhow!("Invalid encode type. Valid types are: STANDARD, STANDARD_NO_PAD, URL_SAFE_PAD, URL_SAFE_NO_PAD"))
    };
    decode_type.decode(content.as_bytes()).map(|res| String::from_utf8_lossy(&res).to_string()).map_err(|e| anyhow!("Error decoding base64: {:?}", e))
}
