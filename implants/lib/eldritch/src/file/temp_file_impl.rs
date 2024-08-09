use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

pub fn temp_file(name: String) -> Result<String> {
    //create a file in temp folder
    let tmp_file = NamedTempFile::new()?;
    let tdir: PathBuf = tmp_file.path().parent().unwrap().into();
    let new_path = tdir.join(name);
    let (_tf, tpath) = tmp_file.keep()?;

    fs::rename(&tpath, &new_path)?;

    Ok(String::from(new_path.to_str().unwrap()).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_temp_file() -> anyhow::Result<()> {
        // Create file
        let p = temp_file("foo".to_string())?;
        // check if file exists
        assert!(Path::new(&p).exists());

        Ok(())
    }
    #[test]
    fn test_temp_no_file() -> anyhow::Result<()> {
        // Create file and then delete it (so we know it doesnt exist)
        let p = temp_file("foo".to_string())?;
        if Path::new(&p).exists() {
            // delete the file
            fs::remove_file(&p)?;
        }

        // check if file exists
        assert!(!Path::new(&p).exists());

        Ok(())
    }
}
