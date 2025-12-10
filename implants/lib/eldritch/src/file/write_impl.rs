use anyhow::{anyhow, Result};
use std::{fs::File, io::Write};

pub fn write(path: String, content: String) -> Result<()> {
    let mut f = File::create(&path).map_err(|err| anyhow!("File could not be created: {err}"))?;
    f.write_all(content.as_bytes())
        .map_err(|err| anyhow!("Failed to write to file: {err}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek};

    use super::*;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_write_file_success() -> anyhow::Result<()> {
        // Write to a file where we know it doesn't already exist
        let tempdir = tempdir()?;
        let path = tempdir
            .path()
            .join("writetest.txt")
            .to_string_lossy()
            .to_string();

        assert!(write(path, "Hello World!".to_string()).is_ok());
        tempdir.close()?;
        Ok(())
    }

    #[test]
    fn test_write_file_exists() -> anyhow::Result<()> {
        // Attempt to write to a file that already exists
        let mut tmp_file = NamedTempFile::new()?;
        tmp_file.write_all("this is a very very very long string".as_bytes())?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();

        assert!(write(path, "Hello World!".to_string()).is_ok());
        let mut tmp_str = String::new();
        // be kind and rewind!
        tmp_file.seek(std::io::SeekFrom::Start(0))?;
        tmp_file.read_to_string(&mut tmp_str)?;
        assert_eq!(tmp_str, "Hello World!".to_string());
        tmp_file.close()?;
        Ok(())
    }

    #[test]
    fn test_write_fail_directory_exists() -> anyhow::Result<()> {
        // Attempt to write to a file that is currently a directory and fail
        let dir = tempdir()?;
        let path = String::from(dir.path().to_str().unwrap());

        assert!(write(path, "Hello World!".to_string()).is_err());
        Ok(())
    }
}
