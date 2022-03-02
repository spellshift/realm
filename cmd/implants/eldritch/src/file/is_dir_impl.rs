use anyhow::Result;
use std::path::Path;

pub fn is_dir(path: String) -> Result<bool> {
    let res = Path::new(&path);
    return Ok(res.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::remove_file;

    #[test]
    fn test_is_dir_true() -> anyhow::Result<()>{
        let winfile = "/tmp/";

        // Run our code
        let res = is_dir(String::from(winfile))?;

        assert_eq!(res, true);
        Ok(())
    }
    #[test]
    fn test_is_dir_file() -> anyhow::Result<()>{
        let winfile = "/tmp/win_test_is_dir_file";
        let _ = remove_file(String::from(winfile));
        // Create file
        let mut file = File::create(winfile)?;
        // Write to file
        file.write_all(b"aoeu")?;

        // Run our code
        let res = is_dir(String::from(winfile))?;

        assert_eq!(res, false);

        remove_file(String::from(winfile))?;
        Ok(())
    }
    // Make sure non-existent error is thrown
    #[test]
    fn test_is_dir_nonexistent() -> anyhow::Result<()>{
        let winfile = "/tmp/win_test_is_dir_nonexistent";
        let _ = remove_file(String::from(winfile));

        // Run our code. Don't bubble up errors since we know it will error.
        let res = is_dir(String::from(winfile));
        assert_eq!(res.unwrap(), false);
        Ok(())
    }
}
