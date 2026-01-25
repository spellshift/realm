use alloc::format;
use alloc::string::{String, ToString};
use base64::{Engine, engine::general_purpose};

pub fn decode_b64(content: String, encode_type: Option<String>) -> Result<String, String> {
    let decode_type = match encode_type
        .unwrap_or_else(|| "STANDARD".to_string())
        .as_str()
    {
        "STANDARD" => general_purpose::STANDARD,
        "STANDARD_NO_PAD" => general_purpose::STANDARD_NO_PAD,
        "URL_SAFE" => general_purpose::URL_SAFE,
        "URL_SAFE_NO_PAD" => general_purpose::URL_SAFE_NO_PAD,
        _ => {
            return Err(
                "Invalid encode type. Valid types are: STANDARD, STANDARD_NO_PAD, URL_SAFE_PAD, URL_SAFE_NO_PAD"
                    .into(),
            )
        }
    };
    decode_type
        .decode(content.as_bytes())
        .map(|res| String::from_utf8_lossy(&res).to_string())
        .map_err(|e| format!("Error decoding base64: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_b64() -> Result<(), String> {
        let res = decode_b64("dGVzdA==".to_string(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "test");
        let res = decode_b64("dGVzdA".to_string(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "test");
        let res = decode_b64(
            "aHR0cHM6Ly9nb29nbGUuY29tLyY=".to_string(),
            Some("URL_SAFE".to_string()),
        )?;
        assert_eq!(res, "https://google.com/&");
        let res = decode_b64(
            "aHR0cHM6Ly9nb29nbGUuY29tLyY".to_string(),
            Some("URL_SAFE_NO_PAD".to_string()),
        )?;
        assert_eq!(res, "https://google.com/&");
        Ok(())
    }

    #[test]
    fn test_decode_b64_invalid_type() {
        let res = decode_b64("test".to_string(), Some("INVALID".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_decode_b64_default_type() -> Result<(), String> {
        let res = decode_b64("dGVzdA==".to_string(), None)?;
        assert_eq!(res, "test");
        Ok(())
    }

    #[test]
    fn test_decode_b64_invalid_content() {
        let res = decode_b64("///".to_string(), Some("STANDARD".to_string()));
        assert!(res.is_err());
    }
}
