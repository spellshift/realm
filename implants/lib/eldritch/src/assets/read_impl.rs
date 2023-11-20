use anyhow::Result;

pub fn read(src: String) -> Result<String> {
    let src_file_bytes = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };
    let mut result = String::new();
    for byte in src_file_bytes.iter() {
        result.push(*byte as char);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_read() -> anyhow::Result<()> {
        let res = read("print/main.eld".to_string())?;
        assert_eq!(res, r#"print("This script just prints")"#);
        Ok(())
    }
}
