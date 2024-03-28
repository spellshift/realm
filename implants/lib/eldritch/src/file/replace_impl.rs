use anyhow::Result;
use regex::bytes::{NoExpand, Regex};
use std::fs::{self, write};

pub fn replace(path: String, pattern: String, value: String) -> Result<()> {
    let data = fs::read(path.clone())?;
    let file_contents_bytes = data.as_slice();
    let re = Regex::new(&pattern)?;
    let result = re.replace(file_contents_bytes, NoExpand(value.as_bytes()));
    write(path, result)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_replace_multiline() -> anyhow::Result<()> {
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(
            path_new.clone(),
            "Match User anoncvs\nMatch User anoncvs\nMatch User anoncvs\n",
        );

        // Run our code
        replace(
            path_new.clone(),
            String::from("Match"),
            String::from("Not Match"),
        )?;

        let res = read_to_string(path_new)?;
        assert_eq!(
            res,
            "Not Match User anoncvs\nMatch User anoncvs\nMatch User anoncvs\n"
        );
        Ok(())
    }
    #[test]
    fn test_replace_regex_simple() -> anyhow::Result<()> {
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(
            path_new.clone(),
            "Match User anoncvs\nMatch User anoncvs\nMatch User anoncvs\n",
        );

        // Run our code
        replace(
            path_new.clone(),
            String::from(r".*Match.*"),
            String::from("Not Match"),
        )?;

        let res = read_to_string(path_new)?;
        assert_eq!(res, "Not Match\nMatch User anoncvs\nMatch User anoncvs\n");
        Ok(())
    }
    #[test]
    fn test_replace_regex_binary() -> anyhow::Result<()> {
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = "/tmp/test.bin".to_string(); //String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(
            path_new.clone(),
            b"\x90\x90\x90\x90\x90\x90\x90\x90\x90AAAAAAAA\x91\x91\x91\x91\x91\x91\x91\x91\x91",
        );

        // Run our code
        replace(
            path_new.clone(),
            String::from("AAAAAAAA"),
            String::from("BBBBBBBB"),
        )?;

        let data = fs::read(path_new.clone())?;
        let file_contents_bytes = data.as_slice();
        assert_eq!(
            file_contents_bytes,
            b"\x90\x90\x90\x90\x90\x90\x90\x90\x90BBBBBBBB\x91\x91\x91\x91\x91\x91\x91\x91\x91"
        );
        Ok(())
    }
    #[test]
    fn test_replace_regex_complex() -> anyhow::Result<()> {
        let tmp_file_new = NamedTempFile::new()?;
        let path_new = String::from(tmp_file_new.path().to_str().unwrap()).clone();
        let _ = write(
            path_new.clone(),
            "MaxStartups 10:30:100\nListenAddress 0.0.0.0\nMaxAuthTries 6\nMatch User anoncvs\n",
        );

        // Run our code
        replace(
            path_new.clone(),
            String::from(r"\d\.\d\.\d\.\d"),
            String::from("127.0.0.1"),
        )?;

        let res = read_to_string(path_new)?;
        assert_eq!(
            res,
            "MaxStartups 10:30:100\nListenAddress 127.0.0.1\nMaxAuthTries 6\nMatch User anoncvs\n"
        );
        Ok(())
    }
}
