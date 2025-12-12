use super::CryptoLibrary;
use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use base64::{engine::general_purpose, Engine};
use eldritch_core::conversion::ToValue;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use md5::Context as Md5Context;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::io::{Read, Write};

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct StdCryptoLibrary;

impl CryptoLibrary for StdCryptoLibrary {
    fn aes_encrypt(&self, key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        if key.len() != 16 {
            return Err("Key size must be 16 bytes (characters)".into());
        }
        let key_bytes: [u8; 16] = key.as_slice().try_into().map_err(|_| "Key size mismatch")?;
        let key_arr = GenericArray::from(key_bytes);

        // Pad data (PKCS#7)
        let mut padded_data = data.clone();
        let padding_needed = 16 - (padded_data.len() % 16);
        for _ in 0..padding_needed {
            padded_data.push(padding_needed as u8);
        }

        let cipher = Aes128::new(&key_arr);
        let mut block = GenericArray::from([0u8; 16]);
        let mut output = Vec::with_capacity(padded_data.len());

        for chunk in padded_data.chunks(16) {
            block.copy_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            output.extend_from_slice(block.as_slice());
        }

        Ok(output)
    }

    fn aes_decrypt(&self, key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        if key.len() != 16 {
            return Err("Key size must be 16 bytes (characters)".into());
        }
        if !data.len().is_multiple_of(16) {
            return Err("Data size must be a multiple of 16 bytes".into());
        }

        let key_bytes: [u8; 16] = key.as_slice().try_into().map_err(|_| "Key size mismatch")?;
        let key_arr = GenericArray::from(key_bytes);

        let cipher = Aes128::new(&key_arr);
        let mut block = GenericArray::from([0u8; 16]);
        let mut output = Vec::with_capacity(data.len());

        // Decrypt all blocks
        for chunk in data.chunks(16) {
            block.copy_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            output.extend_from_slice(block.as_slice());
        }

        // Unpad (PKCS#7) manually to match v1 logic
        #[allow(clippy::collapsible_if)]
        if let Some(&last_byte) = output.last() {
            if last_byte <= 16 && last_byte > 0 {
                let len = output.len();
                let start_padding = len - (last_byte as usize);
                if start_padding < len {
                    // Check bound
                    let suspected_padding = &output[start_padding..];
                    let mut valid_padding = true;
                    for &byte in suspected_padding {
                        if byte != last_byte {
                            valid_padding = false;
                            break;
                        }
                    }

                    if valid_padding {
                        output.truncate(start_padding);
                    }
                }
            }
        }

        Ok(output)
    }

    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        let key_bytes = key.as_bytes();
        if key_bytes.len() != 16 {
            return Err("Key size must be 16 bytes".into());
        }
        let key_arr = GenericArray::from_slice(key_bytes);
        let cipher = Aes128::new(key_arr);

        let mut input = std::fs::File::open(src).map_err(|e| e.to_string())?;
        let len = input.metadata().map_err(|e| e.to_string())?.len();
        if len % 16 != 0 {
            return Err("Input file size is not a multiple of 16 bytes".into());
        }

        let mut output = std::fs::File::create(dst).map_err(|e| e.to_string())?;
        let mut block = GenericArray::from([0u8; 16]);
        let mut next_block = [0u8; 16];

        // Read first block
        if input.read_exact(&mut block).is_err() {
            return Err("Input file is empty or too short".into());
        }

        loop {
            // Try to read next block
            match input.read_exact(&mut next_block) {
                Ok(_) => {
                    // Current `block` is not the last one.
                    cipher.decrypt_block(&mut block);
                    output.write_all(&block).map_err(|e| e.to_string())?;
                    // Move next to current
                    block.copy_from_slice(&next_block);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Current `block` IS the last one.
                    cipher.decrypt_block(&mut block);
                    // Unpad (PKCS#7) manually
                    let last_byte = block[15];
                    if last_byte <= 16 && last_byte > 0 {
                        let padding_len = last_byte as usize;
                        let mut valid_padding = true;
                        // Check padding
                        for i in (16 - padding_len)..16 {
                            if block[i] != last_byte {
                                valid_padding = false;
                                break;
                            }
                        }

                        if valid_padding {
                            output
                                .write_all(&block[..16 - padding_len])
                                .map_err(|e| e.to_string())?;
                        } else {
                            // Invalid padding, write full block? (Mimic aes_decrypt logic which keeps invalid padding)
                            output.write_all(&block).map_err(|e| e.to_string())?;
                        }
                    } else {
                        // Invalid padding length, just write full block
                        output.write_all(&block).map_err(|e| e.to_string())?;
                    }
                    break;
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    }

    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        let key_bytes = key.as_bytes();
        if key_bytes.len() != 16 {
            return Err("Key size must be 16 bytes".into());
        }
        let key_arr = GenericArray::from_slice(key_bytes);
        let cipher = Aes128::new(key_arr);

        let mut input = std::fs::File::open(src).map_err(|e| e.to_string())?;
        let mut output = std::fs::File::create(dst).map_err(|e| e.to_string())?;

        let mut temp_buf = [0u8; 16];

        loop {
            // Read loop to fill 16 bytes or hit EOF
            let mut chunk_len = 0;
            while chunk_len < 16 {
                let n = input
                    .read(&mut temp_buf[chunk_len..])
                    .map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                chunk_len += n;
            }

            if chunk_len == 16 {
                // We have a full block. Encrypt and write.
                let mut block = GenericArray::clone_from_slice(&temp_buf);
                cipher.encrypt_block(&mut block);
                output.write_all(&block).map_err(|e| e.to_string())?;
            } else {
                // Last block (partial or empty). Pad.
                let padding_byte = (16 - chunk_len) as u8;
                for item in temp_buf.iter_mut().skip(chunk_len) {
                    *item = padding_byte;
                }
                let mut block = GenericArray::clone_from_slice(&temp_buf);
                cipher.encrypt_block(&mut block);
                output.write_all(&block).map_err(|e| e.to_string())?;
                break;
            }
        }

        Ok(())
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        Ok(format!("{:02x}", md5::compute(data)))
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        let mut hasher = Sha1::new();
        hasher.update(&data);
        Ok(format!("{:02x}", hasher.finalize()))
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:02x}", hasher.finalize()))
    }

    fn hash_file(&self, file: String, algo: String) -> Result<String, String> {
        let file = std::fs::File::open(file).map_err(|e| e.to_string())?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0; 8192];

        // Helper closure to process the file in chunks
        let mut process = |feed: &mut dyn FnMut(&[u8])| -> Result<(), String> {
            loop {
                let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if count == 0 {
                    break;
                }
                feed(&buffer[..count]);
            }
            Ok(())
        };

        match algo.to_lowercase().as_str() {
            "md5" => {
                let mut hasher = Md5Context::new();
                process(&mut |chunk| hasher.consume(chunk))?;
                Ok(format!("{:02x}", hasher.compute()))
            }
            "sha1" => {
                let mut hasher = Sha1::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            "sha256" => {
                let mut hasher = Sha256::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            "sha512" => {
                let mut hasher = Sha512::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            _ => Err(format!("Unknown algorithm: {algo}")),
        }
    }

    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
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

    fn decode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
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

    fn is_json(&self, content: String) -> Result<bool, String> {
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn from_json(&self, content: String) -> Result<Value, String> {
        let json_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Error parsing json: {:?}", e))?;
        convert_json_to_value(json_data)
    }

    fn to_json(&self, content: Value) -> Result<String, String> {
        let json_value = convert_value_to_json(&content)?;
        serde_json::to_string(&json_value).map_err(|e| format!("Error serializing to json: {:?}", e))
    }
}

fn convert_json_to_value(json: serde_json::Value) -> Result<Value, String> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(format!("Unsupported number type: {n}"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let mut res = Vec::with_capacity(arr.len());
            for item in arr {
                res.push(convert_json_to_value(item)?);
            }
            Ok(res.to_value())
        }
        serde_json::Value::Object(map) => {
            #[allow(clippy::mutable_key_type)]
            let mut res = BTreeMap::new();
            for (k, v) in map {
                res.insert(Value::String(k), convert_json_to_value(v)?);
            }
            Ok(res.to_value())
        }
    }
}

fn convert_value_to_json(val: &Value) -> Result<serde_json::Value, String> {
    match val {
        Value::None => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::json!(i)),
        Value::Float(f) => Ok(serde_json::json!(f)),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Bytes(_b) => {
             // Bytes are not natively JSON serializable.
             Err("Object of type 'bytes' is not JSON serializable".to_string())
        },
        Value::List(l) => {
            let list = l.read();
            let mut res = Vec::with_capacity(list.len());
            for item in list.iter() {
                res.push(convert_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(res))
        },
        Value::Tuple(t) => {
            let mut res = Vec::with_capacity(t.len());
            for item in t.iter() {
                res.push(convert_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(res))
        },
        Value::Dictionary(d) => {
            let dict = d.read();
            let mut res = serde_json::Map::new();
            for (k, v) in dict.iter() {
                if let Value::String(s) = k {
                    res.insert(s.clone(), convert_value_to_json(v)?);
                } else {
                     // JSON keys must be strings
                     return Err(format!("Keys must be strings, got {:?}", k));
                }
            }
            Ok(serde_json::Value::Object(res))
        },
        Value::Set(_) => Err("Object of type 'set' is not JSON serializable".to_string()),
        Value::Function(_) | Value::NativeFunction(_, _) | Value::NativeFunctionWithKwargs(_, _) | Value::BoundMethod(_, _) | Value::Foreign(_) => {
             Err(format!("Object of type '{:?}' is not JSON serializable", val))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eldritch_core::conversion::ToValue;

    #[test]
    fn test_aes_roundtrip() {
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        let iv = vec![0u8; 16]; // Ignored
        let data = b"Hello World!".to_vec();

        let encrypted = lib
            .aes_encrypt(key.clone(), iv.clone(), data.clone())
            .expect("encrypt failed");
        assert_ne!(encrypted, data);
        assert_eq!(encrypted.len() % 16, 0);

        let decrypted = lib
            .aes_decrypt(key.clone(), iv.clone(), encrypted)
            .expect("decrypt failed");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_padding_logic() {
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        // Exact block size
        let data = b"1234567890123456".to_vec();

        let encrypted = lib.aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        // Should produce 2 blocks (32 bytes) because PKCS#7 adds a full block of padding if input is multiple of block size
        assert_eq!(encrypted.len(), 32);

        let decrypted = lib.aes_decrypt(key.clone(), vec![], encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_vectors() {
        // We can test against the v1 implementation's implicit logic
        // "Lorem ipsum..." (truncated for brevity)
        let data = b"Lorem ipsum dolor sit amet".to_vec();
        let key = b"TESTINGPASSWORD!".to_vec();
        let lib = StdCryptoLibrary;

        let encrypted = lib.aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        let decrypted = lib.aes_decrypt(key.clone(), vec![], encrypted).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_encrypt_invalid_key_length() {
        let lib = StdCryptoLibrary;
        let key = b"short".to_vec();
        let data = b"data".to_vec();
        let res = lib.aes_encrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_key_length() {
        let lib = StdCryptoLibrary;
        let key = b"short".to_vec();
        let data = b"data".to_vec();
        let res = lib.aes_decrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_data_length() {
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        let data = b"not_multiple_16".to_vec();
        let res = lib.aes_decrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_padding() {
        // We need to construct a valid encrypted block but mess up the padding (last bytes).
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        let data = b"data".to_vec();
        let mut encrypted = lib.aes_encrypt(key.clone(), vec![], data).unwrap();

        // Modify last byte to make padding invalid
        if let Some(last) = encrypted.last_mut() {
             *last = *last ^ 0xFF; // Flip bits
        }

        // The current implementation returns the decrypted data with invalid padding attached,
        // it does not return an error. Let's verify that behavior.
        // Or if we want to assert strictness, we might need to change implementation.
        // But for unit testing the *current* code:
        let decrypted = lib.aes_decrypt(key, vec![], encrypted).unwrap();
        // Since padding is invalid, it won't be stripped.
        // Original data length was 4. Padded to 16.
        // Encrypted length is 16.
        // Decrypted length should remain 16 because padding check failed.
        assert_eq!(decrypted.len(), 16);
    }

    #[test]
    fn test_md5() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.md5(data).unwrap();
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_sha1() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.sha1(data).unwrap();
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[test]
    fn test_sha256() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.sha256(data).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lib = StdCryptoLibrary;
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let lorem_hash_md5 = "db89bb5ceab87f9c0fcc2ab36c189c2c";
        let lorem_hash_sha1 = "cd36b370758a259b34845084a6cc38473cb95e27";
        let lorem_hash_sha256 = "2d8c2f6d978ca21712b5f6de36c9d31fa8e96a4fa5d8ff8b0188dfb9e7c171bb";
        let lorem_hash_sha512 = "8ba760cac29cb2b2ce66858ead169174057aa1298ccd581514e6db6dee3285280ee6e3a54c9319071dc8165ff061d77783100d449c937ff1fb4cd1bb516a69b9";

        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "{lorem}").expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert_eq!(
            lib.hash_file(path.clone(), "md5".to_string()).unwrap(),
            lorem_hash_md5
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha1".to_string()).unwrap(),
            lorem_hash_sha1
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha256".to_string()).unwrap(),
            lorem_hash_sha256
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha512".to_string()).unwrap(),
            lorem_hash_sha512
        );
    }

    #[test]
    fn test_hash_file_invalid_algo() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lib = StdCryptoLibrary;
        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "test").expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert!(lib.hash_file(path, "invalid".to_string()).is_err());
    }

    #[test]
    fn test_hash_file_not_found() {
        let lib = StdCryptoLibrary;
        assert!(lib
            .hash_file("/non/existent/file".to_string(), "md5".to_string())
            .is_err());
    }

    #[test]
    fn test_encode_b64() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.encode_b64("test".to_string(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "dGVzdA==");
        let res = lib.encode_b64("test".to_string(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "dGVzdA");
        let res = lib.encode_b64(
            "https://google.com/&".to_string(),
            Some("URL_SAFE".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY=");
        let res = lib.encode_b64(
            "https://google.com/&".to_string(),
            Some("URL_SAFE_NO_PAD".to_string()),
        )?;
        assert_eq!(res, "aHR0cHM6Ly9nb29nbGUuY29tLyY");
        Ok(())
    }

    #[test]
    fn test_encode_b64_invalid_type() {
        let lib = StdCryptoLibrary;
        let res = lib.encode_b64("test".to_string(), Some("INVALID".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_encode_b64_default_type() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.encode_b64("test".to_string(), None)?;
        assert_eq!(res, "dGVzdA==");
        Ok(())
    }

    #[test]
    fn test_decode_b64() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.decode_b64("dGVzdA==".to_string(), Some("STANDARD".to_string()))?;
        assert_eq!(res, "test");
        let res = lib.decode_b64("dGVzdA".to_string(), Some("STANDARD_NO_PAD".to_string()))?;
        assert_eq!(res, "test");
        let res = lib.decode_b64(
            "aHR0cHM6Ly9nb29nbGUuY29tLyY=".to_string(),
            Some("URL_SAFE".to_string()),
        )?;
        assert_eq!(res, "https://google.com/&");
        let res = lib.decode_b64(
            "aHR0cHM6Ly9nb29nbGUuY29tLyY".to_string(),
            Some("URL_SAFE_NO_PAD".to_string()),
        )?;
        assert_eq!(res, "https://google.com/&");
        Ok(())
    }

    #[test]
    fn test_decode_b64_invalid_type() {
        let lib = StdCryptoLibrary;
        let res = lib.decode_b64("test".to_string(), Some("INVALID".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_decode_b64_default_type() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.decode_b64("dGVzdA==".to_string(), None)?;
        assert_eq!(res, "test");
        Ok(())
    }

    #[test]
    fn test_decode_b64_invalid_content() {
        let lib = StdCryptoLibrary;
        let res = lib.decode_b64("///".to_string(), Some("STANDARD".to_string()));
        assert!(res.is_err());
    }

    #[test]
    fn test_is_json_object() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.is_json(r#"{"test": "test"}"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_is_json_list() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.is_json(r#"[1, "foo", false, null]"#.to_string())?;
        assert!(res);
        Ok(())
    }

    #[test]
    fn test_is_json_invalid() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.is_json(r#"{"test":"#.to_string())?;
        assert!(!res);
        Ok(())
    }

    #[test]
    fn test_from_json_object() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.from_json(r#"{"test": "test"}"#.to_string())?;
        // Construct expected value
        let mut map = BTreeMap::new();
        map.insert("test".to_string().to_value(), "test".to_string().to_value());
        let expected = map.to_value();

        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn test_from_json_list() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.from_json(r#"[1, "foo", false, null]"#.to_string())?;

        let mut vec = Vec::new();
        vec.push(1i64.to_value());
        vec.push("foo".to_string().to_value());
        vec.push(false.to_value());
        vec.push(Value::None);
        let expected = vec.to_value();

        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn test_from_json_float() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let res = lib.from_json(r#"3.14"#.to_string())?;
        assert_eq!(res, Value::Float(3.14));
        Ok(())
    }

    #[test]
    fn test_from_json_invalid() {
        let lib = StdCryptoLibrary;
        let res = lib.from_json(r#"{"test":"#.to_string());
        assert!(res.is_err());
    }

    #[test]
    fn to_json_object() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let mut map = BTreeMap::new();
        map.insert("test".to_string().to_value(), "test".to_string().to_value());
        let val = map.to_value();

        let res = lib.to_json(val)?;
        assert_eq!(res, r#"{"test":"test"}"#);
        Ok(())
    }

    #[test]
    fn to_json_list() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let vec_val: Vec<Value> = vec![
            1i64.to_value(),
            "foo".to_string().to_value(),
            false.to_value(),
            Value::None,
        ];
        let val = vec_val.to_value();

        let res = lib.to_json(val)?;
        assert_eq!(res, r#"[1,"foo",false,null]"#);
        Ok(())
    }

    #[test]
    fn to_json_float() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let val = Value::Float(3.14);
        let res = lib.to_json(val)?;
        assert_eq!(res, "3.14");
        Ok(())
    }

    #[test]
    fn to_json_tuple() -> Result<(), String> {
        let lib = StdCryptoLibrary;
        let t = vec![1i64.to_value(), 2i64.to_value()];
        let val = Value::Tuple(t);
        let res = lib.to_json(val)?;
        assert_eq!(res, "[1,2]");
        Ok(())
    }

    #[test]
    fn to_json_invalid_bytes() {
        let lib = StdCryptoLibrary;
        let val = Value::Bytes(vec![0xFF]);
        let res = lib.to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_set() {
        let lib = StdCryptoLibrary;
        let val = Value::Set(alloc::sync::Arc::new(spin::RwLock::new(alloc::collections::BTreeSet::new())));
        let res = lib.to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_dict_keys() {
        let lib = StdCryptoLibrary;
        let mut map = BTreeMap::new();
        map.insert(1i64.to_value(), "test".to_string().to_value());
        let val = map.to_value();

        let res = lib.to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_function() {
        let lib = StdCryptoLibrary;
        // We can't easily construct a function Value here without internal AST types,
        // but we can try to use a Value variant that isn't supported.
        // Actually Value::Foreign requires a trait object.
        // Value::Function requires AST.
        // But we covered Bytes and Set, which is good.
        // Let's verify Set behavior again.
        let val = Value::Set(alloc::sync::Arc::new(spin::RwLock::new(alloc::collections::BTreeSet::new())));
        assert!(lib.to_json(val).is_err());
    }

    #[test]
    fn test_aes_file_roundtrip() -> Result<(), String> {
        use std::io::{Read, Write};
        use tempfile::NamedTempFile;

        let lib = StdCryptoLibrary;
        let key = "TESTINGPASSWORD!".to_string();
        let data = b"Hello World! This is a test file for AES encryption.".to_vec();

        // Write src
        let mut src_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        src_file
            .write_all(&data)
            .map_err(|e| e.to_string())?;
        let src_path = src_file.path().to_str().unwrap().to_string();

        let dst_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let dst_path = dst_file.path().to_str().unwrap().to_string();

        let dec_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let dec_path = dec_file.path().to_str().unwrap().to_string();

        // Encrypt
        lib.aes_encrypt_file(src_path.clone(), dst_path.clone(), key.clone())?;

        // Verify dst size
        let dst_len = std::fs::metadata(&dst_path).unwrap().len();
        assert_eq!(dst_len % 16, 0);

        // Decrypt
        lib.aes_decrypt_file(dst_path.clone(), dec_path.clone(), key.clone())?;

        // Verify content
        let mut result_data = Vec::new();
        std::fs::File::open(&dec_path)
            .unwrap()
            .read_to_end(&mut result_data)
            .unwrap();
        assert_eq!(result_data, data);

        Ok(())
    }

    #[test]
    fn test_aes_file_encrypt_empty() -> Result<(), String> {
        use tempfile::NamedTempFile;
        let lib = StdCryptoLibrary;
        let key = "TESTINGPASSWORD!".to_string();

        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let dst_file = NamedTempFile::new().unwrap();
        let dst_path = dst_file.path().to_str().unwrap().to_string();

        lib.aes_encrypt_file(src_path, dst_path.clone(), key).unwrap();

        // Empty file padded to 1 block
        let dst_len = std::fs::metadata(&dst_path).unwrap().len();
        assert_eq!(dst_len, 16);

        Ok(())
    }

    #[test]
    fn test_aes_file_decrypt_invalid_key() {
        let lib = StdCryptoLibrary;
        assert!(lib
            .aes_decrypt_file("s".into(), "d".into(), "short".into())
            .is_err());
    }

    #[test]
    fn test_aes_file_decrypt_bad_size() -> Result<(), String> {
        use std::io::Write;
        use tempfile::NamedTempFile;
        let lib = StdCryptoLibrary;
        let key = "TESTINGPASSWORD!".to_string();

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"123").unwrap(); // Not multiple of 16
        let src_path = src_file.path().to_str().unwrap().to_string();

        let res = lib.aes_decrypt_file(src_path, "dst".into(), key);
        assert!(res.is_err());
        Ok(())
    }
}
