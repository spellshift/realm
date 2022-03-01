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

    #[test]
    fn test_copy() -> std::io::Result<()>{
        // Create file
        let mut file = File::create("/tmp/win_copy1")?;
        // Write to file
        file.write_all(b"Hello, world!")?;

        // Run our code
        let _res = copy(String::from("/tmp/win_copy1"), String::from("/tmp/win_copy2"));

        // Open copied file
        let mut winfile = File::open("/tmp/win_copy2")?;
        // Read
        let mut contents = String::new();
        winfile.read_to_string(&mut contents)?;
        // Compare
        assert_eq!(contents, "Hello, world!");
        Ok(())
    }
}
