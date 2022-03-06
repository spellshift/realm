use anyhow::Result;
use std::path::Path;

pub fn exists(path: String) -> Result<bool> {
    let res = Path::new(&path).exists();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::remove_file;

    #[test]
    fn test_exists_file() -> anyhow::Result<()>{
        let winfile = "/tmp/win_test_exists_does";
        let _ = remove_file(String::from(winfile));
        // Create file
        let mut file = File::create(winfile)?;
        // Write to file
        file.write_all(b"Hello, world!")?;

        // Run our code
        let res = exists(String::from(winfile))?;

        assert_eq!(res, true);

        remove_file(String::from(winfile))?;
        Ok(())
    }
    #[test]
    fn test_exists_no_file() -> anyhow::Result<()>{
        let winfile = "/tmp/win_test_exists_doesnt";
        let _ = remove_file(String::from(winfile));

        // Run our code
        let res = exists(String::from(winfile))?;

        assert_eq!(res, false);

        let _ = remove_file(String::from(winfile));
        Ok(())
    }
    #[test]
    fn test_exists_dir() -> anyhow::Result<()>{
        let winfile = "/tmp/";

        // Run our code
        let res = exists(String::from(winfile))?;

        assert_eq!(res, true);
        Ok(())
    }
    #[test]
    fn test_exists_no_dir() -> anyhow::Result<()>{
        let winfile = "/aoeu/";

        // Run our code
        let res = exists(String::from(winfile))?;

        assert_eq!(res, false);
        Ok(())
    }
}
