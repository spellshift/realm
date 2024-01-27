use anyhow::Result;
use std::path::Path;

pub fn exists(path: String) -> Result<bool> {
    let res = Path::new(&path).exists();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_exists_file() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to file
        tmp_file.write_all(b"Hello, world!")?;

        // Run our code
        let res = exists(path)?;

        assert!(res);

        Ok(())
    }
    #[test]
    fn test_exists_no_file() -> anyhow::Result<()> {
        // Create file and then delete it (so we know it doesnt exist)
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Run our code
        let res = exists(path)?;

        assert!(!res);

        Ok(())
    }
    #[test]
    fn test_exists_dir() -> anyhow::Result<()> {
        // Create Dir
        let dir = tempdir()?;
        let path = String::from(dir.path().to_str().unwrap());

        // Run our code
        let res = exists(path)?;

        assert!(res);
        Ok(())
    }
    #[test]
    fn test_exists_no_dir() -> anyhow::Result<()> {
        // Create Dir and then delete it (so we know it doesnt exist)
        let dir = tempdir()?;
        let path = String::from(dir.path().to_str().unwrap()).clone();
        dir.close()?;

        // Run our code
        let res = exists(path)?;

        assert!(!res);
        Ok(())
    }
}
