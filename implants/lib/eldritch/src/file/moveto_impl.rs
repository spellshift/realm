use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn moveto(old: String, new: String) -> Result<()> {
    // If path is a dir delete it.
    // This will help unify behavior across systems.
    // https://doc.rust-lang.org/std/fs/fn.rename.html#platform-specific-behavior
    if Path::new(&new.clone()).is_dir() {
        fs::remove_dir_all(new.clone())?;
    } else if Path::new(&new.clone()).is_file() {
        fs::remove_file(new.clone())?;
    }

    fs::rename(old, new)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile,tempdir};
    use std::path::Path;
    use std::fs::File;

    #[test]
    fn test_moveto_dir() -> anyhow::Result<()>{
        let tmp_dir = tempdir()?;
        let path_old = String::from(tmp_dir.path().to_str().unwrap()).clone();
        let tmp_dir_new = tempdir()?;
        let path_new = String::from(tmp_dir_new.path().to_str().unwrap()).clone();
        tmp_dir_new.close()?;


        moveto(path_old.clone(), path_new.clone())?;

        assert_eq!(Path::new(&path_old).is_dir(), false);
        assert_eq!(Path::new(&path_new).is_dir(), true);
        Ok(())
    }
    #[test]
    fn test_moveto_file() -> anyhow::Result<()>{
        let tmp_file = NamedTempFile::new()?;
        let path_old = String::from(tmp_file.path().to_str().unwrap()).clone();
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        tmp_file_new.close()?;
        // Run our code

        moveto(path_old.clone(), path_new.clone())?;

        assert_eq!(Path::new(&path_old).is_file(), false);
        assert_eq!(Path::new(&path_new).is_file(), true);
        Ok(())
    }
    #[test]
    fn test_moveto_file_overwrites() -> anyhow::Result<()>{
        let tmp_file = NamedTempFile::new()?;
        let path_old = String::from(tmp_file.path().to_str().unwrap()).clone();
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        // Run our code

        moveto(path_old.clone(), path_new.clone())?;

        assert_eq!(Path::new(&path_old).is_file(), false);
        assert_eq!(Path::new(&path_new).is_file(), true);
        Ok(())
    }
    #[test]
    fn test_moveto_dir_overwrites() -> anyhow::Result<()>{
        // Create initial directory and file.
        let tmp_dir = tempdir()?;
        let tmp_dir_path = tmp_dir.path();

        let path_old = String::from(tmp_dir_path.to_str().unwrap()).clone();
        let path_old_file = String::from(tmp_dir_path.join("myfile2").to_str().unwrap()).clone();
        let _ = File::create(path_old_file.clone())?;
        let _ = fs::write(path_old_file.clone(),"Hello");

        // Create destination directory and the file path we expect after moveto "win"
        let tmp_dir_new = tempdir()?;
        let path_new = String::from(tmp_dir_new.path().to_str().unwrap()).clone();
        let path_new_file = String::from(tmp_dir_new.path().join("myfile").to_str().unwrap()).clone();
        let path_new_file_win = String::from(tmp_dir_new.path().join("myfile2").to_str().unwrap()).clone();

        let _ = File::create(path_new_file.clone())?;
    

        // Run our code
        moveto(path_old.clone(), path_new.clone())?;

        // Assert
        assert_eq!(Path::new(&path_old).is_dir(), false);
        assert_eq!(Path::new(&path_new_file_win).is_file(), true);
        assert_eq!(Path::new(&path_new_file).is_file(), false);
        assert_eq!(Path::new(&path_new).is_dir(), true);
        Ok(())
    }

}
