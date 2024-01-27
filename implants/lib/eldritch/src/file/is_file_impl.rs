use anyhow::Result;
use std::path::Path;

pub fn is_file(path: String) -> Result<bool> {
    let res = Path::new(&path);
    Ok(res.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_is_file_basic() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to file
        tmp_file.write_all(b"Hello, world!")?;

        // Run our code
        let res = is_file(path)?;

        assert!(res);

        Ok(())
    }

    #[test]
    fn test_is_file_negative() -> anyhow::Result<()> {
        // Create file and then delete it (so we know it doesnt exist)
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Run our code
        let res = is_file(path)?;

        assert!(!res);

        Ok(())
    }

    #[test]
    fn test_is_file_dir() -> anyhow::Result<()> {
        // Create Dir
        let dir = tempdir()?;
        let path = String::from(dir.path().to_str().unwrap());

        // Run our code
        let res = is_file(path)?;

        assert!(!res);
        Ok(())
    }
}
