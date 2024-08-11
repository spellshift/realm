use anyhow::Result;
use std::env;
use std::fs::File;
use tempfile::NamedTempFile;

pub fn temp_file(name: Option<String>) -> Result<String> {
    let mut temp_path;
    let _file;

    match name {
        None => {
            // Generate a random file name if name is not provided
            let tfile = NamedTempFile::new()?;
            (_file, temp_path) = tfile.keep()?;
        }
        Some(n) => {
            temp_path = env::temp_dir();
            temp_path.push(n);
            _file = File::create(&temp_path)?;
        }
    }
    // Create the file in the temporary directory

    Ok(temp_path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_temp_file_w_name() -> anyhow::Result<()> {
        // Create file with a name
        let p = temp_file(Some("foo".to_string()))?;
        // check if file exists
        assert!(Path::new(&p).exists());

        Ok(())
    }
    #[test]
    fn test_temp_file_r_name() -> anyhow::Result<()> {
        // Create file with a random name
        let p = temp_file(None)?;
        // check if file exists
        assert!(Path::new(&p).exists());

        Ok(())
    }
    #[test]
    fn test_temp_no_file_w_name() -> anyhow::Result<()> {
        // Create file with a name and then delete it (so we know it doesnt exist)
        let p = temp_file(Some("foo".to_string()))?;
        if Path::new(&p).exists() {
            // delete the file
            fs::remove_file(&p)?;
        }

        // check file doesn't exists
        assert!(!Path::new(&p).exists());

        Ok(())
    }
    #[test]
    fn test_temp_no_file_r_name() -> anyhow::Result<()> {
        // Create file with a random name and then delete it (so we know it doesnt exist)
        let p = temp_file(None)?;
        if Path::new(&p).exists() {
            // delete the file
            fs::remove_file(&p)?;
        }

        // check file doesn't exists
        assert!(!Path::new(&p).exists());

        Ok(())
    }
}
