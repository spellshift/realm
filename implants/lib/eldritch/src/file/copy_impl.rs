use anyhow::Result;
use std::fs;

pub fn copy(src: String, dst: String) -> Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Write to file
        tmp_file_src.write_all(b"Hello, world!")?;

        // Run our code
        copy(path_src, path_dst)?;

        // Read
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare
        assert_eq!(contents, "Hello, world!");

        Ok(())
    }
}
