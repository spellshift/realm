use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn remove(path: String) -> Result<()> {
    let res = Path::new(&path);
    if res.is_file() {
        fs::remove_file(path)?;
    } else if res.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn remove_file() -> anyhow::Result<()> {
        // Create file
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();

        // Run our code
        remove(path.clone())?;

        // Verify that file has been removed
        let res = Path::new(&path).exists();
        assert_eq!(res, false);
        Ok(())
    }
    #[test]
    fn remove_dir() -> anyhow::Result<()> {
        // Create dir
        let tmp_dir = tempdir()?;
        let path = String::from(tmp_dir.path().to_str().unwrap());

        // Run our code
        remove(path.clone())?;

        // Verify that file has been removed
        let res = Path::new(&path).exists();
        assert_eq!(res, false);
        Ok(())
    }
}
