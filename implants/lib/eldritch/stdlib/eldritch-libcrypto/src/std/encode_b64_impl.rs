use alloc::string::{String, ToString};
use base64::{Engine, engine::general_purpose};

pub fn encode_b64(content: String, encode_type: Option<String>) -> Result<String, String> {
    let encode_type = match encode_type
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
    Ok(encode_type.encode(content.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_b64() -> Result<(), String> {
        let res = encode_b64("test".to_string(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "dGVzdA==");
        let res = encode_b64("test".to_string(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "dGVzdA");
        let res = encode_b64(
            "https://google.com/&".to_string(),
            Some("URL_SAFE".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY=");
        let res = encode_b64(
            "https://google.com/&".to_string(),
            Some("URL_SAFE_NO_PAD".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY");
        Ok(())
    }

    #[test]
    fn test_encode_b64_invalid_type() {
        let res = encode_b64("test".to_string(), Some("INVALID".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_encode_b64_default_type() -> Result<(), String> {
        let res = encode_b64("test".to_string(), None)?;
        assert_eq!(res, "dGVzdA==");
        Ok(())
    }
}
