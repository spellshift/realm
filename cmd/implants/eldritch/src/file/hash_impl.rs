use anyhow::Result;
use sha256::digest_file;

pub fn hash(path: String) -> Result<String> {
    let val = digest_file(path).unwrap();
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::fs::remove_file;

    #[test]
    fn test_hash() -> anyhow::Result<()>{
        let winfile = "/tmp/win_test_exists_does";
        let _ = remove_file(String::from(winfile));
        // Create file
        let mut file = File::create(winfile)?;
        // Write to file
        file.write_all(b"aoeu")?;

        // Run our code
        let res = hash(String::from(winfile))?;

        assert_eq!(res, "bc4c24181ed3ce6666444deeb95e1f61940bffee70dd13972beb331f5d111e9b");

        remove_file(String::from(winfile))?;
        Ok(())
    }
}