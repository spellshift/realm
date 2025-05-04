use crate::runtime::{messages::AsyncMessage, messages::FetchAssetMessage, Environment};
use anyhow::{Context, Result};
use pb::c2::FetchAssetResponse;
use starlark::{eval::Evaluator, values::list::ListRef};
use std::{
    fs,
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::mpsc::{channel, Receiver},
};

fn copy_local(src: String, dst: String) -> Result<()> {
    let src_file = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };

    match fs::write(dst, src_file) {
        Ok(_) => Ok(()),
        Err(local_err) => Err(anyhow::anyhow!(local_err)),
    }
}

fn copy_remote(rx: Receiver<FetchAssetResponse>, dst_path: String) -> Result<()> {
    // Wait for our first chunk
    let resp = rx.recv()?;

    // Delete file if it exists
    if Path::new(&dst_path).exists() {
        fs::remove_file(&dst_path).context("failed to delete existing file")?;
    }

    // Open file for writing
    let mut dst = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dst_path)
        .context(format!("failed to open file for writing: {}", &dst_path))?;

    // Write our first chunk
    dst.write_all(&resp.chunk)
        .context(format!("failed to write file chunk: {}", &dst_path))?;

    // Listen for more chunks and write them
    for resp in rx {
        dst.write_all(&resp.chunk)
            .context(format!("failed to write file chunk: {}", &dst_path))?;
    }

    // Ensure all chunks gets written
    dst.flush()
        .context(format!("failed to flush file: {}", &dst_path))?;

    Ok(())
}

// #[allow(clippy::needless_pass_by_ref_mut)]
pub fn copy(starlark_eval: &Evaluator<'_, '_>, src: String, dst: String) -> Result<()> {
    let remote_assets = starlark_eval.module().get("remote_assets");

    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("`remote_assets` is not type list")?;
        let src_value = starlark_eval.module().heap().alloc_str(&src);

        if tmp_list.contains(&src_value.to_value()) {
            let env = Environment::from_extra(starlark_eval.extra)?;
            let (tx, rx) = channel();
            env.send(AsyncMessage::from(FetchAssetMessage { name: src, tx }))?;

            return copy_remote(rx, dst);
        }
    }
    copy_local(src, dst)
}

#[cfg(test)]
mod tests {
    use crate::{
        assets::copy_impl::copy_remote,
        runtime::messages::{AsyncMessage, FetchAssetMessage, Message, ReportErrorMessage},
    };
    use pb::c2::FetchAssetResponse;
    use pb::eldritch::Tome;
    use std::{collections::HashMap, io::prelude::*};
    use std::{fs, sync::mpsc::channel};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_remote_copy() -> anyhow::Result<()> {
        // Create files
        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());
        let check_dst = path_dst.clone();

        let (tx, rx) = channel();
        let handle =
            tokio::task::spawn_blocking(|| copy_remote(rx, path_dst).expect("copy_remote failed"));

        tx.send(FetchAssetResponse {
            chunk: "Hello from a remote asset".as_bytes().to_vec(),
        })?;
        tx.send(FetchAssetResponse {
            chunk: "Goodbye from a remote asset".as_bytes().to_vec(),
        })?;

        // Drop the Sender, to indicate no more data will be sent (channel closed)
        drop(tx);

        handle.await?;

        let contents = fs::read_to_string(check_dst)?;
        assert!(contents.contains("Hello from a remote asset"));
        assert!(contents.contains("Goodbye from a remote asset"));
        Ok(())
    }

    #[tokio::test]
    async fn test_remote_copy_full() -> anyhow::Result<()> {
        // Create files
        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());
        let check_dst = path_dst.clone();

        // Run Eldritch (in it's own thread)
        let mut runtime = crate::start(
            123,
            Tome {
                eldritch: r#"assets.copy("test_tome/test_file.txt", input_params['test_output'])"#
                    .to_owned(),
                parameters: HashMap::from([("test_output".to_string(), path_dst)]),
                file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
            },
        )
        .await;

        // We now mock the agent, looping until eldritch requests a file
        // We omit the sleep performed by the agent, just to save test time
        loop {
            // The runtime only returns the data that is currently available
            // So this may return an empty vec if our eldritch tokio task has not yet been scheduled
            let messages = runtime.collect();
            let mut fetch_asset_msgs: Vec<&FetchAssetMessage> = messages
                .iter()
                .filter_map(|m| match m {
                    Message::Async(AsyncMessage::FetchAsset(fam)) => Some(fam),
                    _ => None,
                })
                .collect();

            // If no asset request is yet available, just continue looping
            if fetch_asset_msgs.is_empty() {
                continue;
            }

            // Ensure the right asset was requested
            assert!(fetch_asset_msgs.len() == 1);
            let msg = fetch_asset_msgs.pop().expect("no asset request received!");
            assert!(msg.name == "test_tome/test_file.txt");

            // Now, we provide the file to eldritch (as a series of chunks)
            msg.tx
                .send(FetchAssetResponse {
                    chunk: "chunk1\n".as_bytes().to_vec(),
                })
                .expect("failed to send file chunk to eldritch");
            msg.tx
                .send(FetchAssetResponse {
                    chunk: "chunk2\n".as_bytes().to_vec(),
                })
                .expect("failed to send file chunk to eldritch");

            // We've finished providing the file, so we stop looping
            // This will drop `req`, which consequently drops the underlying `Sender` for the file channel
            // This will cause the next `recv()` to error with "channel is empty and sending half is closed"
            // which is what tells eldritch that there are no more file chunks to wait for
            break;
        }

        // Now that we've finished writing data, we wait for eldritch to finish
        runtime.finish().await;

        // Lastly, assert the file was written correctly
        let contents = fs::read_to_string(check_dst)?;
        assert_eq!("chunk1\nchunk2\n", contents.as_str());

        Ok(())
    }

    #[tokio::test]
    async fn test_embedded_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let path_src = "exec_script/hello_world.sh".to_string();
        #[cfg(target_os = "windows")]
        let path_src = "exec_script/hello_world.bat".to_string();

        let runtime = crate::start(
            123,
            Tome {
                eldritch: r#"assets.copy(input_params['src_file'], input_params['test_output'])"#
                    .to_owned(),
                parameters: HashMap::from([
                    ("src_file".to_string(), path_src),
                    ("test_output".to_string(), path_dst),
                ]),
                file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
            },
        )
        .await;

        let messages = runtime.collect();
        let errors = messages
            .iter()
            .filter_map(|m| match m {
                Message::Async(AsyncMessage::ReportError(rem)) => Some(rem),
                _ => None,
            })
            .collect::<Vec<&ReportErrorMessage>>();
        assert!(errors.is_empty());

        let mut contents: String = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        assert!(contents.contains("hello from an embedded shell script"));

        Ok(())
    }
}
