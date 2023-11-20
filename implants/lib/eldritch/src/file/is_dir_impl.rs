use anyhow::Result;
use std::path::Path;

pub fn is_dir(path: String) -> Result<bool> {
    let res = Path::new(&path);
    return Ok(res.is_dir());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_is_dir_true() -> anyhow::Result<()> {
        let tmp_dir = tempdir()?;
        let path = String::from(tmp_dir.path().to_str().unwrap()).clone();
        // Run our code

        let res = is_dir(path)?;

        assert_eq!(res, true);
        Ok(())
    }
    #[test]
    fn test_is_dir_file() -> anyhow::Result<()> {
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        tmp_file.write_all(b"Hello, world!\n")?;

        // Run our code
        let res = is_dir(path)?;

        assert_eq!(res, false);
        Ok(())
    }
    // Make sure non-existent error is thrown
    #[test]
    fn test_is_dir_nonexistent() -> anyhow::Result<()> {
        let tmp_dir = tempdir()?;
        let path = String::from(
            tmp_dir
                .path()
                .join("win_test_is_dir_nonexistent")
                .to_str()
                .unwrap(),
        )
        .clone();
        // Run our code

        let res = is_dir(path)?;

        assert_eq!(res, false);
        Ok(())
    }
}
