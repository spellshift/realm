use anyhow::Result;
use glob::{glob, GlobError};
use std::{fs, path::PathBuf};

pub fn read_binary(path: String) -> Result<Vec<u32>> {
    let mut res: Vec<u8> = Vec::new();
    let glob_res = glob(&path)?.collect::<Vec<Result<PathBuf, GlobError>>>();
    if glob_res.is_empty() {
        return Err(anyhow::anyhow!(
            "file.read_binary: pattern {} found no results",
            path,
        ));
    }

    for entry in glob_res {
        match entry {
            Ok(entry_path) => {
                let data = fs::read(entry_path)?;
                res.extend(data);
            }
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::debug!("Failed to parse glob {}\n{}", path, _err);
            }
        }
    }
    Ok(res.into_iter().map(|b| b as u32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::prelude::*};
    use tempfile::{tempdir, NamedTempFile};

    const HELLO_WORLD_STR_BYTES: &[u8; 14] = b"Hello, world!\n";
    const HELLO_WORLD_BYTES: [u32; 14] = [
        0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x0a,
    ];

    #[test]
    fn test_read_simple() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        tmp_file.write_all(HELLO_WORLD_STR_BYTES)?;

        // Run our code
        let res = read_binary(path)?;
        // Verify output
        assert_eq!(res, Vec::from(HELLO_WORLD_BYTES));
        Ok(())
    }
    #[test]
    fn test_read_large() -> anyhow::Result<()> {
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());

        // Write to New File
        for _ in 0..256 {
            tmp_file.write_all(HELLO_WORLD_STR_BYTES)?;
        }

        // Run our code
        let res = read_binary(path)?;
        let expected = HELLO_WORLD_BYTES.repeat(256);
        // Verify output
        assert_eq!(res, expected);
        Ok(())
    }
    #[test]
    fn test_read_nonexistent() -> anyhow::Result<()> {
        // Create file
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        tmp_file.close()?;

        let res = read_binary(path);
        assert!(res.is_err());
        Ok(())
    }

    fn contains_slice<T: PartialEq>(vec: &[T], slice: &[T]) -> bool {
        vec.windows(slice.len()).any(|window| window == slice)
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
        let res = read_binary(path)?;

        assert!(!contains_slice(&res, &[0x42, 0x79, 0x65]));
        assert_eq!(
            res,
            Vec::from([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x0a, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x0a])
        );
        Ok(())
    }
}
