use anyhow::{Context, Result};
use starlark::const_frozen_string;
use starlark::eval::Evaluator;
use std::fs;

pub fn copy<'v>(starlark_eval: &mut Evaluator<'v, '_>, src: String, dst: String) -> Result<()> {
    // Check remote
    let remote_assets = starlark_eval.module().get("remote_assets");
    match remote_assets {
        Some(assets) => {
            let json_string = assets.to_json()?;
            println!("{}", json_string);
            let json_map: serde_json::Value = serde_json::from_str(&json_string)?;
            match json_map.get(src.clone()) {
                Some(remote_src) => {
                    println!("{}", remote_src)
                }
                None => {}
            }
        }
        None => {}
    }

    // Check local
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
    use crate::insert_dict_kv;

    use super::*;
    use starlark::{
        collections::SmallMap,
        environment::Module,
        values::{dict::Dict, AllocValue, Value},
    };
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remote_copy() -> anyhow::Result<()> {
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module);

        let res: SmallMap<Value, Value> = SmallMap::new();
        let mut remote_assets: Dict = Dict::new(res);
        insert_dict_kv!(
            remote_assets,
            module.heap(),
            "test_tome/test_file.txt",
            "grpc://remote/file",
            String
        );

        module.set("remote_assets", remote_assets.alloc_value(module.heap()));

        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        copy(&mut eval, "test_tome/test_file.txt".to_string(), path_dst)?;
        #[cfg(any(target_os = "windows"))]
        copy(&mut eval, "test_tome/test_file.txt".to_string(), path_dst)?;

        // Read
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare
        assert!(contents.contains("hello from an embedded shell script"));

        Ok(())
    }

    #[test]
    fn test_embedded_copy() -> anyhow::Result<()> {
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module);

        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        copy(
            &mut eval,
            "exec_script/hello_world.sh".to_string(),
            path_dst,
        )?;
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
