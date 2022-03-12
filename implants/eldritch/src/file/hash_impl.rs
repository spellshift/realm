use anyhow::Result;
use sha256::digest_file;

pub fn hash(path: String) -> Result<String> {
    let val = digest_file(path)?;
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash() -> anyhow::Result<()>{
        // Create file
        let mut tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap());
        
        // Write to file
        tmp_file.write_all(b"aoeu")?;

        // Run our code
        let res = hash(path)?;

        assert_eq!(res, "bc4c24181ed3ce6666444deeb95e1f61940bffee70dd13972beb331f5d111e9b");

        Ok(())
    }
}