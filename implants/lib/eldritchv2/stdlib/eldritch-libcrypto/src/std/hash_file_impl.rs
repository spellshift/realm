use alloc::format;
use alloc::string::{String, ToString};
use md5::Context as Md5Context;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::io::Read;

pub fn hash_file(file: String, algo: String) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let lorem_hash_md5 = "db89bb5ceab87f9c0fcc2ab36c189c2c";
        let lorem_hash_sha1 = "cd36b370758a259b34845084a6cc38473cb95e27";
        let lorem_hash_sha256 = "2d8c2f6d978ca21712b5f6de36c9d31fa8e96a4fa5d8ff8b0188dfb9e7c171bb";
        let lorem_hash_sha512 = "8ba760cac29cb2b2ce66858ead169174057aa1298ccd581514e6db6dee3285280ee6e3a54c9319071dc8165ff061d77783100d449c937ff1fb4cd1bb516a69b9";

        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "{lorem}").expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert_eq!(
            hash_file(path.clone(), "md5".to_string()).unwrap(),
            lorem_hash_md5
        );
        assert_eq!(
            hash_file(path.clone(), "sha1".to_string()).unwrap(),
            lorem_hash_sha1
        );
        assert_eq!(
            hash_file(path.clone(), "sha256".to_string()).unwrap(),
            lorem_hash_sha256
        );
        assert_eq!(
            hash_file(path.clone(), "sha512".to_string()).unwrap(),
            lorem_hash_sha512
        );
    }

    #[test]
    fn test_hash_file_invalid_algo() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "test").expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert!(hash_file(path, "invalid".to_string()).is_err());
    }

    #[test]
    fn test_hash_file_not_found() {
        assert!(
            hash_file("/non/existent/file".to_string(), "md5".to_string())
                .is_err()
        );
    }
}
