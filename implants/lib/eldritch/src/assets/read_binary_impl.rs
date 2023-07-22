use anyhow::Result;

pub fn read_binary(src: String) -> Result<Vec<u32>> {
    let src_file_bytes = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };
    let result = src_file_bytes.iter().map(|x| *x as u32).collect::<Vec<u32>>();
    // let mut result = Vec::new();
    // for byt: Vec<u32>e in src_file_bytes.iter() {
    //     result.push(*byte as u32);
    // }
    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_assets_read_binary() -> anyhow::Result<()>{
        let res = read_binary("print/main.eld".to_string())?;
        assert_eq!(res, [112,114,105,110,116,40,34,84,104,105,115,32,115,99,114,105,112,116,32,106,117,115,116,32,112,114,105,110,116,115,34,41]);
        Ok(())
    }
}
