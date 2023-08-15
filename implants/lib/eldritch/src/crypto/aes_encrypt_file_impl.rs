use std::fs::File;
use std::io::{BufReader, Read, Write};

use anyhow::Result;
use crypto::aes::{self, KeySize};
use crypto::symmetriccipher::SynchronousStreamCipher;

pub fn encrypt_file(src: String, dst: String, key: String) -> Result<()> {
    let bytes = key.as_bytes();
    let mut cipher = aes::ctr(KeySize::KeySize128, bytes, bytes);
    let f = File::open(src)?;
    let mut reader = BufReader::new(f);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let mut output: Vec<u8> = Vec::with_capacity(buffer.len());
    for _ in 0..output.capacity() {
        output.push(0);
    }
    cipher.process(&buffer, &mut output[..]);
    let mut f = File::create(dst)?;
    f.write_all(output.as_slice())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::{Write, Read}};

    use tempfile::NamedTempFile;
    use anyhow::Result;

    use super::encrypt_file;

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        
        let tmp_file_enc = NamedTempFile::new()?;
        let path_enc = String::from(tmp_file_enc.path().to_str().unwrap()).clone();

        let tmp_file_dec = NamedTempFile::new()?;
        let path_dec = String::from(tmp_file_dec.path().to_str().unwrap()).clone();
        {
            let mut f = File::create(path.clone())?;
            write!(f, "{}", lorem)?;
        }
        encrypt_file(path, path_enc.clone(), "TESTING".to_string())?;
        encrypt_file(path_enc, path_dec.clone(), "TESTING".to_string())?;

        let mut dec_f = File::open(path_dec)?;
        let mut dec_f_content = Vec::new();
        dec_f.read_to_end(&mut dec_f_content)?;
        assert_eq!(String::from_utf8_lossy(&dec_f_content).to_string().as_str(), lorem);
        Ok(())
    }

    #[test]
    fn test_encrypt_no_file() -> Result<()> {
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        assert!(encrypt_file("/I/Dont/Exist".to_string(), path.clone(), "TESTING".to_string()).is_err());
        assert!(encrypt_file(path.clone(), "/I/Dont/Exist".to_string(), "TESTING".to_string()).is_err());
        Ok(())
    }
}
