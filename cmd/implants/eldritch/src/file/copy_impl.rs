use anyhow::Result;
use std::fs;

pub fn copy(src: String, dst: String) -> Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::remove_file;

    #[test]
    fn test_copy() -> anyhow::Result<()>{
        let _ = remove_file(String::from("/tmp/win_copy1"));
        let _ = remove_file(String::from("/tmp/win_copy2"));

        // Create file
        let mut file = File::create("/tmp/win_copy1")?;
        // Write to file
        file.write_all(b"Hello, world!")?;

        // Run our code
        copy(String::from("/tmp/win_copy1"), String::from("/tmp/win_copy2"))?;

        // Open copied file
        let mut winfile = File::open("/tmp/win_copy2")?;
        // Read
        let mut contents = String::new();
        winfile.read_to_string(&mut contents)?;
        // Compare
        assert_eq!(contents, "Hello, world!");

        remove_file(String::from("/tmp/win_copy1"))?;
        remove_file(String::from("/tmp/win_copy2"))?;
        Ok(())
    }
}
