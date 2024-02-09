use crate::runtime::Client;
use anyhow::{Context, Result};
use starlark::{eval::Evaluator, values::list::ListRef};
use std::{fs, sync::mpsc::Receiver};

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
        let val = match file_reciever.recv() {
            Ok(v) => v,
            Err(err) => {
                match err.to_string().as_str() {
                    "channel is empty and sending half is closed" => {
                        break;
                    }
                    "timed out waiting on channel" => {
                        continue;
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        log::debug!("failed to drain channel: {}", err)
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

    use std::{collections::HashMap, io::prelude::*};
    use tempfile::NamedTempFile;

    // fn init_log() {
    //     pretty_env_logger::formatted_timed_builder()
    //         .filter_level(log::LevelFilter::Info)
    //         .parse_env("IMIX_LOG")
    //         .init();
    // }

    // #[tokio::test]
    // async fn test_remote_copy() -> anyhow::Result<()> {
    //     // Create files
    //     let mut tmp_file_dst = NamedTempFile::new()?;
    //     let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

    //     let (sender, reciver) = channel::<Vec<u8>>();
    //     sender.send("Hello from a remote asset".as_bytes().to_vec())?;

    //     copy_remote(reciver, path_dst)?;

    //     let mut contents = String::new();
    //     tmp_file_dst.read_to_string(&mut contents)?;
    //     assert!(contents.contains("Hello from a remote asset"));
    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_remote_copy_full() -> anyhow::Result<()> {
    //     init_log();
    //     log::debug!("Testing123");

    //     // Create files
    //     let mut tmp_file_dst = NamedTempFile::new()?;
    //     let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

    //     let (runtime, broker) = Runtime::new();
    //     let handle = tokio::task::spawn_blocking(move || {
    //         runtime.run(crate::pb::Tome {
    //             eldritch: r#"assets.copy("test_tome/test_file.txt", input_params['test_output'])"#
    //                 .to_owned(),
    //             parameters: HashMap::from([("test_output".to_string(), path_dst)]),
    //             file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
    //         })
    //     });
    //     handle.await?;
    //     println!("{:?}", broker.collect_file_requests().len());
    //     assert!(broker.collect_errors().is_empty()); // No errors even though the remote asset is inaccessible

    //     let mut contents = String::new();
    //     tmp_file_dst.read_to_string(&mut contents)?;
    //     // Compare - Should be empty basically just didn't error
    //     assert!(contents.contains(""));

    //     Ok(())
    // }

    #[test]
    fn test_embedded_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let path_src = "exec_script/hello_world.sh".to_string();
        #[cfg(target_os = "windows")]
        let path_src = "exec_script/hello_world.bat".to_string();

        let (runtime, broker) = Runtime::new();
        runtime.run(crate::pb::Tome {
            eldritch: r#"assets.copy(input_params['src_file'], input_params['test_output'])"#
                .to_owned(),
            parameters: HashMap::from([
                ("src_file".to_string(), path_src),
                ("test_output".to_string(), path_dst),
            ]),
            file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
        });

        assert!(broker.collect_errors().is_empty()); // No errors even though the remote asset is inaccessible

        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        assert!(contents.contains("hello from an embedded shell script"));

        Ok(())
    }
}
