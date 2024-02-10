use crate::runtime::Client;
use anyhow::{Context, Result};
use starlark::{eval::Evaluator, values::list::ListRef};
use std::fs::OpenOptions;
use std::io::Write;
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

fn copy_remote(rx: Receiver<Vec<u8>>, dst_path: String) -> Result<()> {
    let mut dst = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&dst_path)
        .context(format!("failed to open destination file: {}", &dst_path))?;
    dst.set_len(0)
        .context(format!("failed to truncate existing file: {}", &dst_path))?; // Truncate if existing

    for chunk in rx {
        dst.write_all(&chunk)
            .context(format!("failed to write file chunk: {}", &dst_path))?;
    }

    dst.flush()?;

    Ok(())
}

// #[allow(clippy::needless_pass_by_ref_mut)]
pub fn copy(starlark_eval: &Evaluator<'_, '_>, src: String, dst: String) -> Result<()> {
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
    use crate::assets::copy_impl::copy_remote;
    use crate::Runtime;

    use std::sync::mpsc::channel;
    use std::{collections::HashMap, io::prelude::*};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_remote_copy() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        let (ch_data, data) = channel::<Vec<u8>>();
        let handle = tokio::task::spawn_blocking(|| {
            copy_remote(data, path_dst).expect("copy_remote failed")
        });

        ch_data.send("Hello from a remote asset".as_bytes().to_vec())?;
        ch_data.send("Goodbye from a remote asset".as_bytes().to_vec())?;

        // Drop the Sender, to indicate no more data will be sent (channel closed)
        drop(ch_data);

        handle.await?;

        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        assert!(contents.contains("Hello from a remote asset"));
        assert!(contents.contains("Goodbye from a remote asset"));
        Ok(())
    }

    #[tokio::test]
    async fn test_remote_copy_full() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Create a runtime
        let (runtime, broker) = Runtime::new();

        // Execute eldritch in it's own thread
        let handle = tokio::task::spawn_blocking(move || {
            runtime.run(crate::pb::Tome {
                eldritch: r#"assets.copy("test_tome/test_file.txt", input_params['test_output'])"#
                    .to_owned(),
                parameters: HashMap::from([("test_output".to_string(), path_dst)]),
                file_names: Vec::from(["test_tome/test_file.txt".to_string()]),
            })
        });

        // We now mock the agent, looping until eldritch requests a file
        // We omit the sleep performed by the agent, just to save test time
        loop {
            // The broker only returns the data that is currently available
            // So this may return an empty vec if our eldritch tokio task has not yet been scheduled
            let mut reqs = broker.collect_file_requests();

            // If no file request is yet available, just continue looping
            if reqs.is_empty() {
                continue;
            }

            // Ensure the right file was requested
            assert!(reqs.len() == 1);
            let req = reqs.pop().expect("no file request received!");
            assert!(req.name() == "test_tome/test_file.txt");

            // Now, we provide the file to eldritch (as a series of chunks)
            req.send_chunk("chunk1\n".as_bytes().to_vec())
                .expect("failed to send file chunk to eldritch");
            req.send_chunk("chunk2\n".as_bytes().to_vec())
                .expect("failed to send file chunk to eldritch");

            // We've finished providing the file, so we stop looping
            // This will drop `req`, which consequently drops the underlying `Sender` for the file channel
            // This will cause the next `recv()` to error with "channel is empty and sending half is closed"
            // which is what tells eldritch that there are no more file chunks to wait for
            break;
        }

        // Now that we've finished writing data, we wait for eldritch to finish
        handle.await?;

        // Lastly, assert the file was written correctly
        let mut contents = String::new();
        tmp_file_dst.read_to_string(&mut contents)?;
        assert_eq!("chunk1\nchunk2\n", contents.as_str());

        Ok(())
    }

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
