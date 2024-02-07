use anyhow::{Context, Result};
use starlark::{eval::Evaluator, values::list::ListRef};
use std::fs;

use crate::Runtime;

pub fn copy(starlark_eval: &mut Evaluator<'_, '_>, src: String, dst: String) -> Result<()> {
    let remote_assets = starlark_eval.module().get("remote_assets");
    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("Assets is not type list")?;
        let src_value = starlark_eval.module().heap().alloc_str(&src);
        if tmp_list.contains(&src_value.to_value()) {
            println!("{}", src);
            let eld_runtime = Runtime::from_extra(starlark_eval.extra)?;
            eld_r
            return Ok(());
        }
    }

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
    use starlark::{environment::Module, values::AllocValue};
    use std::io::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remote_copy() -> anyhow::Result<()> {
        let module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&module);

        module.set(
            "remote_assets",
            Vec::from(["test_tome/test_file.txt".to_string()]).alloc_value(module.heap()),
        );

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
        #[cfg(target_os = "windows")]
        copy("exec_script/hello_world.bat".to_string(), path_dst)?;

        // Read
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare
        assert!(contents.contains("hello from an embedded shell script"));

        Ok(())
    }
}
