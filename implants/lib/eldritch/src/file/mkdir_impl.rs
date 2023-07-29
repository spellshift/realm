use anyhow::Result;
use std::fs;

pub fn mkdir(path: String) -> Result<()> {
    fs::create_dir(&path).map_err(|_| anyhow::anyhow!(format!("Failed to create directory at path: {}", path)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use anyhow::Ok;
    use tempfile::tempdir;
    use std::path::Path;

    #[test]
    fn test_successful_mkdir() -> Result<()> {
        let tmp_dir_parent = tempdir()?;
        let path_dir = String::from(tmp_dir_parent.path().to_str().unwrap()).clone();
        tmp_dir_parent.close()?;
        
        let result = mkdir(path_dir.clone());
        assert!(result.is_ok(), "Expected mkdir to succeed, but it failed: {:?}", result);

        let binding = path_dir.clone();
        let res = Path::new(&binding);
        assert!(res.is_dir(), "Directory not created successfully.");

        fs::remove_dir_all(path_dir.clone()).ok();

        Ok(())
    } 

    #[test]
    fn test_error_mkdir() -> Result<()>{
        let tmp_dir_parent = tempdir()?;
        let path_dir = String::from(tmp_dir_parent.path().to_str().unwrap()).clone();
        tmp_dir_parent.close()?;

        let result = mkdir(format!("{}/{}", path_dir, "dir".to_string()));

        assert!(
            result.is_err(),
            "Expected mkdir to fail, but it succeeded: {:?}",
            result
        );

        Ok(())
    }
}