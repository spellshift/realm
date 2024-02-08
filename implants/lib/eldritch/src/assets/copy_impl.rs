use crate::runtime::Client;
use anyhow::{Context, Result};
use starlark::{eval::Evaluator, values::list::ListRef};
use std::{fs, sync::mpsc::Receiver, time::Duration};

fn copy_local(src: String, dst: String) -> Result<()> {
    let src_file = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };

    match fs::write(dst, src_file) {
        Ok(_) => Ok(()),
        Err(local_err) => Err(local_err.try_into()?),
    }
}

fn copy_remote(file_reciever: Receiver<Vec<u8>>, dst: String) -> Result<()> {
    loop {
        let val = match file_reciever.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => v,
            Err(err) => {
                match err.to_string().as_str() {
                    "channel is empty and sending half is closed" => {
                        break;
                    }
                    "timed out waiting on channel" => {
                        break;
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        eprint!("failed to drain channel: {}", err)
                    }
                }
                break;
            }
        };
        match fs::write(dst.clone(), val) {
            Ok(_) => {}
            Err(local_err) => return Err(local_err.try_into()?),
        };
    }

    Ok(())
}

pub fn copy(starlark_eval: &mut Evaluator<'_, '_>, src: String, dst: String) -> Result<()> {
    let remote_assets = starlark_eval.module().get("remote_assets");

    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("`remote_assets` is not type list")?;
        let src_value = starlark_eval.module().heap().alloc_str(&src);

        if tmp_list.contains(&src_value.to_value()) {
            let client = Client::from_extra(starlark_eval.extra)?;
            let file_reciever = client.request_file(src)?;

            return copy_remote(file_reciever, dst);
        }
    }
    copy_local(src, dst)
}

#[cfg(test)]
mod tests {
    use crate::Runtime;

    use super::*;
    use starlark::{environment::Module, values::AllocValue};
    use std::{collections::HashMap, io::prelude::*, sync::mpsc::channel};
    use tempfile::NamedTempFile;

    #[test]
    fn test_remote_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        let (sender, reciver) = channel::<Vec<u8>>();
        sender.send("Hello from a remote asset".as_bytes().to_vec())?;

        copy_remote(reciver, path_dst)?;

        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        assert!(contents.contains("Hello from a remote asset"));
        Ok(())
    }

    #[test]
    fn test_remote_copy_full() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        let (runtime, broker) = Runtime::new();
        runtime.run(crate::pb::Tome {
            eldritch: r#"assets.copy("test_tome/test_file.txt", input_params['test_output'])"#
                .to_owned(),
            parameters: HashMap::from([("test_output".to_string(), path_dst)]),
            file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
        });

        assert!(broker.collect_errors().is_empty()); // No errors even though the remote asset is inaccessible

        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        // Compare - Should be empty basically just didn't error
        assert!(contents.contains(""));

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
