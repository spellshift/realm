use anyhow::{anyhow, Result};
use base64::{Engine, engine::general_purpose};

pub fn encode_b64(content: String, encode_type: Option<String>) -> Result<String> {
    let encode_type = match encode_type.unwrap_or("STANDARD".to_string()).as_str() {
        "STANDARD" => {general_purpose::STANDARD},
        "STANDARD_NO_PAD" => {general_purpose::STANDARD_NO_PAD},
        "URL_SAFE" => {general_purpose::URL_SAFE},
        "URL_SAFE_NO_PAD" => {general_purpose::URL_SAFE_NO_PAD},
        _ => return Err(anyhow!("Invalid encode type. Valid types are: STANDARD, STANDARD_NO_PAD, URL_SAFE_PAD, URL_SAFE_NO_PAD"))
    };
    Ok(encode_type.encode(content.as_bytes()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_encode_b64() -> anyhow::Result<()>{
        let res = super::encode_b64("test".to_string(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "dGVzdA==");
        let res = super::encode_b64("test".to_string(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "dGVzdA");
        let res = super::encode_b64("https://google.com/&".to_string(), Some("URL_SAFE".to_string()))?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY=");
        let res = super::encode_b64("https://google.com/&".to_string(), Some("URL_SAFE_NO_PAD".to_string()))?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY");
        Ok(())
    }

    #[test]
    fn test_encode_b64_invalid_type() -> anyhow::Result<()>{
        let res = super::encode_b64("test".to_string(), Some("INVALID".to_string()));
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_encode_b64_default_type() -> anyhow::Result<()>{
        let res = super::encode_b64("test".to_string(), None)?;
        assert_eq!(res, "dGVzdA==");
        Ok(())
    }
}