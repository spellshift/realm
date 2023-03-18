use anyhow::Result;
use std::fs;

pub fn copy(src: String, dst: String, ) -> Result<()> {
    let index_html = Asset::get(src)?;
    fs::copy(src, dst)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use tempfile::NamedTempFile;
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "../../tests/embedded_files_test"]
    pub struct Asset;
    
    #[test]
    fn test_embedded_copy() -> anyhow::Result<()>{
        let my_script = Asset::get("exec_script/hello_word.sh")?;

    
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
echo "hello from an embedded shell script"#);

        Ok(())
    }
}
