use anyhow::Result;
use glob::glob;
use std::fs;

pub fn read(path: String) -> Result<String> {
    let mut res: String = String::from("");
    for entry in glob(&path)? {
        match entry {
            Ok(enty_path) => {
                let data = fs::read_to_string(enty_path)?;
                res.push_str(data.as_str());
            }
            Err(local_err) => {
                #[cfg(debug_assertions)]
                log::debug!("Failed to parse glob {}\n{}", path, local_err);
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

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
        assert!(res.is_err());
        Ok(())
    }
}
