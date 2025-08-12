use anyhow::Result;
use glob::{glob, GlobError};
use std::{fs, path::PathBuf};

pub fn read(path: String) -> Result<String> {
    let mut res: String = String::from("");
    let glob_res = glob(&path)?.collect::<Vec<Result<PathBuf, GlobError>>>();
    if glob_res.is_empty() {
        return Err(anyhow::anyhow!(
            "file.read: pattern {} found no results",
            path,
        ));
    }

    for entry in glob_res {
        match entry {
            Ok(entry_path) => {
                let data = fs::read(entry_path)?;
                res.push_str(&String::from_utf8_lossy(&data));
            }
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::debug!("Failed to parse glob {}\n{}", path, _err);
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::prelude::*};
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_read_simple() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        tmp_file.write_all(b"Hello, world!\n")?;

        // Run our code
        let res = read(path)?;
        // Verify output
        assert_eq!(res, "Hello, world!\n");
        Ok(())
    }
    #[test]
    fn test_read_large() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        for _ in 0..256 {
            tmp_file.write_all(b"Hello, world!\n")?;
        }

        // Run our code
        let res = read(path)?;
        // Verify output
        assert_eq!(res, "Hello, world!\n".repeat(256));
        Ok(())
    }
    #[test]
    fn test_read_nonexistent() -> anyhow::Result<()> {
        // Create file
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        tmp_file.close()?;

        let res = read(path);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_read_glob() -> anyhow::Result<()> {
        // Create file
        let tmp_dir = tempdir()?;
        let matched_files = ["thesshfile", "anothersshfile"];
        let unmatched_files = ["noswordshere"];
        let tmp_path = tmp_dir.keep();
        for f in matched_files {
            let mut file = File::create(tmp_path.clone().join(f).clone())?;
            file.write_all(b"Hello\n")?;
        }
        for f in unmatched_files {
            let mut file = File::create(tmp_path.clone().join(f))?;
            file.write_all(b"Bye")?;
        }

        let path = String::from(tmp_path.clone().join("*ssh*").to_str().unwrap());
        let res = read(path)?;

        assert!(!res.contains("Bye"));
        assert_eq!(res, "Hello\nHello\n");
        Ok(())
    }
}
