use anyhow::Result;
use rand::{thread_rng, Rng};
use std::io::Write;
use tempfile::NamedTempFile;

pub fn temp_file(path: String) -> Result<String> {
    let mut tmp_file = NamedTempFile::new_in(path)?;
    let mut rng = thread_rng();
    let random_data: Vec<u8> = (0..100).map(|_| rng.gen()).collect();
    tmp_file.write_all(&random_data)?;

    let (_file, path) = tmp_file.keep()?;
    Ok(String::from(path.to_str().unwrap()).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_temp_file() -> anyhow::Result<()> {
        // Create file
        let p = temp_file("".to_string())?;
        // Run our code
        assert!(Path::new(&p).exists());

        Ok(())
    }
    #[test]
    fn test_temp_no_file() -> anyhow::Result<()> {
        // Create file and then delete it (so we know it doesnt exist)
        let p = temp_file("".to_string())?;
        if Path::new(&p).exists() {
            // delete the file
            fs::remove_file(&p)?;
        }

        // Run our code
        assert!(!Path::new(&p).exists());

        Ok(())
    }
}
