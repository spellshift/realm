use anyhow::Result;
use std::fs;

pub fn mkdir(path: String, parent: Option<bool>) -> Result<()> {
    let mut parent_val = false;
    if let Some(parent_some) = parent {
        parent_val = parent_some;
    }
    let res = if parent_val {
        fs::create_dir_all(&path)
    } else {
        fs::create_dir(&path)
    };
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!(format!(
            "Failed to create directory at path: {}",
            path
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_successful_mkdir() -> Result<()> {
        let tmp_dir_parent = tempdir()?;
        let path_dir = String::from(tmp_dir_parent.path().to_str().unwrap()).clone();
        tmp_dir_parent.close()?;

        let result = mkdir(path_dir.clone(), Some(false));
        assert!(
            result.is_ok(),
            "Expected mkdir to succeed, but it failed: {:?}",
            result
        );

        let binding = path_dir.clone();
        let res = Path::new(&binding);
        assert!(res.is_dir(), "Directory not created successfully.");

        fs::remove_dir_all(path_dir.clone()).ok();

        Ok(())
    }

    #[test]
    fn test_mkdir_with_parent() -> Result<()> {
        let tmp_dir_parent = tempdir()?;
        let path_dir = String::from(tmp_dir_parent.path().to_str().unwrap()).clone();
        tmp_dir_parent.close()?;

        let result = mkdir(format!("{}/{}", path_dir, "dir"), Some(true));

        assert!(
            result.is_ok(),
            "Expected mkdir to succeed, but it failed: {:?}",
            result
        );

        Ok(())
    }

    #[test]
    fn test_error_mkdir() -> Result<()> {
        let tmp_dir_parent = tempdir()?;
        let path_dir = String::from(tmp_dir_parent.path().to_str().unwrap()).clone();
        tmp_dir_parent.close()?;

        let result = mkdir(format!("{}/{}", path_dir, "dir"), Some(false));

        assert!(
            result.is_err(),
            "Expected mkdir to fail, but it succeeded: {:?}",
            result
        );

        Ok(())
    }
}
