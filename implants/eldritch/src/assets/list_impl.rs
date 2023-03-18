use std::fs;
use anyhow::Result;

pub fn copy(src: String, dst: String, ) -> Result<()> {
    let src_file = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };
    
    match fs::write(dst, src_file) {
        Ok(_) => Ok(()),
        Err(local_err) => Err(local_err.into()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_embedded_copy() -> anyhow::Result<()>{
   
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code
        copy("exec_script/hello_word.sh".to_string(), path_dst)?;

        // Read
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare
        assert_eq!(contents, r#"#!/bin/sh
echo "hello from an embedded shell script""#);

        Ok(())
    }
}
