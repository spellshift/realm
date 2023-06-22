use anyhow::Result;
use std::fs;

pub fn read(path: String) -> Result<String> {
    let data = fs::read_to_string(path)?;
    return Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::prelude::*;

    #[test]
    fn test_read_simple() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        tmp_file.write_all(b"Hello, world!\n")?;

        // Run our code
        let res = read(path)?;
        // Verify output
        assert_eq!(res, "Hello, world!\n");
        Ok(())
    }    
    #[test]
    fn test_read_large() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        for _ in 0..256 {
            tmp_file.write_all(b"Hello, world!\n")?;
        }

        // Run our code
        let res = read(path)?;
        // Verify output
        assert_eq!(res, "Hello, world!\n".repeat(256));
        Ok(())
    }    
    #[test]
    fn test_read_nonexistent() -> anyhow::Result<()> {
        // Create file
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        tmp_file.close()?;

        let res = read(path);
        assert_eq!(res.is_err(), true);
        Ok(())
    }    
}
