use alloc::string::{String, ToString};
use alloc::vec::Vec;
use base64::{Engine, engine::general_purpose};

pub fn encode_b64(content: Vec<u8>, encode_type: Option<String>) -> Result<String, String> {
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
                "Invalid encode type. Valid types are: STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD"
                    .into(),
            )
        }
    };
    Ok(encode_type.encode(&content))
}

#[cfg(test)]
mod tests {
    use super::super::decode_impl::decode_b64;
    use super::*;

    #[test]
    fn test_encode_b64() -> Result<(), String> {
        let res = encode_b64(b"test".to_vec(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "dGVzdA==");
        let res = encode_b64(b"test".to_vec(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "dGVzdA");
        let res = encode_b64(
            b"https://google.com/&".to_vec(),
            Some("URL_SAFE".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY=");
        let res = encode_b64(
            b"https://google.com/&".to_vec(),
            Some("URL_SAFE_NO_PAD".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY");
        Ok(())
    }

    #[test]
    fn test_encode_b64_invalid_type() {
        let err = encode_b64(b"test".to_vec(), Some("INVALID".to_string())).unwrap_err();
        assert!(err.contains("URL_SAFE,"));
        assert!(!err.contains("URL_SAFE_PAD"));
    }

    #[test]
    fn test_encode_b64_default_type() -> Result<(), String> {
        let res = encode_b64(b"test".to_vec(), None)?;
        assert_eq!(res, "dGVzdA==");
        Ok(())
    }

    #[test]
    fn test_encode_b64_binary_roundtrip() -> Result<(), String> {
        let raw: Vec<u8> = (0u8..=255).collect();
        let encoded = encode_b64(raw.clone(), None)?;
        let decoded = decode_b64(encoded, None)?;
        assert_eq!(decoded, raw);
        Ok(())
    }
}
