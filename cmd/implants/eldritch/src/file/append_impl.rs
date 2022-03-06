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
    use std::io::BufReader;
    use std::fs::File;
    use std::fs::remove_file;

    #[test]
    fn test_append_nonexisting() -> anyhow::Result<()> {
        // In Case Cleanup failed
        let _ = remove_file(String::from("/tmp/win"));

        // Run  our code
        append(String::from("/tmp/win"), String::from("Hi2!"))?;

        // Read the file
        let file = BufReader::new(File::open("/tmp/win")?);

        // Get Last Line
        let last_line = file.lines().last().unwrap()?;

        // Make sure the last line equals == Hi2!
        assert_eq!(last_line, "Hi2!");

        // Cleanup
        remove_file(String::from("/tmp/win"))?;
        Ok(())
    }    
    #[test]
    fn test_append_existing() -> anyhow::Result<()> {
        // In Case Cleanup failed
        let _ = remove_file(String::from("/tmp/win"));

        // Make New File
        let mut file = File::create("/tmp/win")?;
        file.write_all(b"Hello, world!\n")?;

        // Run  our code
        append(String::from("/tmp/win"), String::from("Hi2!"))?;

        // Read the file
        let file = BufReader::new(File::open("/tmp/win")?);

        // Get Last Line
        let last_line = file.lines().last().unwrap()?;

        // Make sure the last line equals == Hi2!
        assert_eq!(last_line, "Hi2!");

        // Cleanup
        let _ = remove_file(String::from("/tmp/win"));
        Ok(())
    }
}
