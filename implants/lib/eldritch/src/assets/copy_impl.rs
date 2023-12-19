use anyhow::Result;
use std::fs;

pub fn copy(src: String, dst: String) -> Result<()> {
    let src_file = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };

    match fs::write(dst, src_file) {
        Ok(_) => Ok(()),
        Err(local_err) => Err(local_err.try_into()?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_embedded_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        copy("exec_script/hello_world.sh".to_string(), path_dst)?;
        #[cfg(any(target_os = "windows"))]
        copy("exec_script/hello_world.bat".to_string(), path_dst)?;

        // Read
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare
        assert!(contents.contains("hello from an embedded shell script"));

        Ok(())
    }
}
