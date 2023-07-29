use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub fn append(path: String, content: String) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true) //Do we want to create the file if it doesn't exist? - Yes!
        .write(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}", content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::BufReader;
    use tempfile::NamedTempFile;

    #[test]
    fn test_append_nonexisting() -> anyhow::Result<()> {
        // Create file and then delete it (so we know it doesnt exist)
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        tmp_file.close()?;

        // Run  our code
        append(path.clone(), String::from("Hi2!"))?;

        // Read the file
        let file_reader = BufReader::new(File::open(path.clone())?);

        // Get Last Line
        let last_line = file_reader.lines().last().unwrap()?;

        // Make sure the last line equals == Hi2!
        assert_eq!(last_line, "Hi2!");

        // Cleanup
        remove_file(path)?;
        Ok(())
    }
    #[test]
    fn test_append_existing() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        tmp_file.write_all(b"Hello, world!\n")?;

        // Run  our code
        append(path, String::from("Hi2!"))?;

        // Read the file
        let file_reader = BufReader::new(tmp_file);

        // Get Last Line
        let last_line = file_reader.lines().last().unwrap()?;

        // Make sure the last line equals == Hi2!
        assert_eq!(last_line, "Hi2!");

        Ok(())
    }
}
