use anyhow::Result;
use std::fs::{write,read_to_string};
use regex::{Regex,NoExpand};

pub fn replace_all(path: String, pattern: String, value: String) -> Result<()> {
    let file_contents = read_to_string(path.clone())?;
    let re = Regex::new(&pattern)?;
    let result = re.replace_all(&file_contents, NoExpand(&value));
    write(path, String::from(result))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile};

    #[test]
    fn test_replace_all_multiline() -> anyhow::Result<()>{
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(path_new.clone(),"Match User anoncvs\nMatch User anoncvs\nMatch User anoncvs\n");
        
        // Run our code
        replace_all(path_new.clone(), String::from("Match"), String::from("Not Match"))?;

        let res = read_to_string(path_new)?;
        assert_eq!(res, "Not Match User anoncvs\n".repeat(3));
        Ok(())
    }
    #[test]
    fn test_replace_all_regex_simple() -> anyhow::Result<()>{
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(path_new.clone(),"Match User anoncvs\nMatch User anoncvs\nMatch User anoncvs\n");
        
        // Run our code
        replace_all(path_new.clone(), String::from(r".*Match.*"), String::from("Not Match"))?;

        let res = read_to_string(path_new)?;
        assert_eq!(res, "Not Match\n".repeat(3));
        Ok(())
    }
    #[test]
    fn test_replace_all_regex_complex() -> anyhow::Result<()>{
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(path_new.clone(),"MaxStartups 10:30:100\nListenAddress 0.0.0.0\nMaxAuthTries 6\nMatch User anoncvs\n");
        
        // Run our code
        replace_all(path_new.clone(), String::from(r"\d\.\d\.\d\.\d"), String::from("127.0.0.1"))?;

        let res = read_to_string(path_new)?;
        assert_eq!(res, "MaxStartups 10:30:100\nListenAddress 127.0.0.1\nMaxAuthTries 6\nMatch User anoncvs\n");
        Ok(())
    }

}