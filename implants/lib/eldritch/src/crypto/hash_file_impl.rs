use std::fs::File;
use std::io::Read;

use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

use anyhow::{anyhow, Result};

pub fn hash_file(file: String, algo: String) -> Result<String> {
    let mut file_data = std::fs::read(file)?;
    match algo.to_lowercase().as_str() {
        "md5" => Ok(format!("{:02x}", md5::compute(file_data))),
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(&file_data);
            Ok(format!("{:02x}", hasher.finalize()))
        }
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(&file_data);
            Ok(format!("{:02x}", hasher.finalize()))
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(&file_data);
            Ok(format!("{:02x}", hasher.finalize()))
        }
        _ => Err(anyhow!("Unknown algorithm: {}", algo)),
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{fs::File, io::Write};

    use tempfile::NamedTempFile;

    use super::hash_file;

    #[test]
    fn test_hash() -> Result<()> {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let lorem_hash_md5 = "db89bb5ceab87f9c0fcc2ab36c189c2c";
        let lorem_hash_sha1 = "cd36b370758a259b34845084a6cc38473cb95e27";
        let lorem_hash_sha256 = "2d8c2f6d978ca21712b5f6de36c9d31fa8e96a4fa5d8ff8b0188dfb9e7c171bb";
        let lorem_hash_sha512 = "8ba760cac29cb2b2ce66858ead169174057aa1298ccd581514e6db6dee3285280ee6e3a54c9319071dc8165ff061d77783100d449c937ff1fb4cd1bb516a69b9";
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        {
            let mut f = File::create(path.clone())?;
            write!(f, "{}", lorem)?;
        }

        assert_eq!(hash_file(path.clone(), "md5".to_string())?, lorem_hash_md5);
        assert_eq!(
            hash_file(path.clone(), "sha1".to_string())?,
            lorem_hash_sha1
        );
        assert_eq!(
            hash_file(path.clone(), "sha256".to_string())?,
            lorem_hash_sha256
        );
        assert_eq!(
            hash_file(path.clone(), "sha512".to_string())?,
            lorem_hash_sha512
        );
        Ok(())
    }

    #[test]
    fn test_hash_invalid() -> Result<()> {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        {
            let mut f = File::create(path.clone())?;
            write!(f, "{}", lorem)?;
        }
        assert!(hash_file(path.clone(), "not_an_algo".to_string()).is_err());
        Ok(())
    }

    #[test]
    fn test_hash_no_file() -> Result<()> {
        assert!(hash_file("/I/Dont/Exist".to_string(), "md5".to_string()).is_err());
        Ok(())
    }
}
