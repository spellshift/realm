use std::sync::mpsc::{channel, Receiver};

use anyhow::{Context, Result};
use pb::c2::FetchAssetResponse;
use starlark::{eval::Evaluator, values::list::ListRef};

use crate::runtime::{messages::AsyncMessage, messages::FetchAssetMessage, Environment};

fn read_binary_remote(rx: Receiver<FetchAssetResponse>) -> Result<Vec<u32>> {
    let mut res: Vec<u32> = vec![];

    // Listen for more chunks and write them
    for resp in rx {
        let mut new_chunk = resp.chunk.iter().map(|x| *x as u32).collect::<Vec<u32>>();
        res.append(&mut new_chunk);
    }

    Ok(res)
}

fn read_binary_embedded(src: String) -> Result<Vec<u32>> {
    let src_file_bytes = match super::Asset::get(src.as_str()) {
        Some(local_src_file) => local_src_file.data,
        None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
    };
    let result = src_file_bytes
        .iter()
        .map(|x| *x as u32)
        .collect::<Vec<u32>>();
    Ok(result)
}

pub fn read_binary(starlark_eval: &Evaluator<'_, '_>, src: String) -> Result<Vec<u32>> {
    let remote_assets = starlark_eval.module().get("remote_assets");

    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("`remote_assets` is not type list")?;
        let src_value = starlark_eval.module().heap().alloc_str(&src);

        if tmp_list.contains(&src_value.to_value()) {
            let env = Environment::from_extra(starlark_eval.extra)?;
            let (tx, rx) = channel();
            env.send(AsyncMessage::from(FetchAssetMessage { name: src, tx }))?;

            return read_binary_remote(rx);
        }
    }
    read_binary_embedded(src)
}

#[cfg(test)]
mod tests {
    use crate::runtime::{messages::AsyncMessage, Message};
    use std::collections::HashMap;

    use crate::runtime::messages::FetchAssetMessage;
    use pb::{c2::FetchAssetResponse, eldritch::Tome};

    use super::*;

    #[test]
    fn test_assets_read_binary() -> anyhow::Result<()> {
        let res = read_binary_embedded("print/main.eldritch".to_string())?;
        #[cfg(not(windows))]
        assert_eq!(
            res,
            [
                112, 114, 105, 110, 116, 40, 34, 84, 104, 105, 115, 32, 115, 99, 114, 105, 112,
                116, 32, 106, 117, 115, 116, 32, 112, 114, 105, 110, 116, 115, 34, 41, 10
            ]
        );
        #[cfg(windows)]
        assert_eq!(
            res,
            [
                112, 114, 105, 110, 116, 40, 34, 84, 104, 105, 115, 32, 115, 99, 114, 105, 112,
                116, 32, 106, 117, 115, 116, 32, 112, 114, 105, 110, 116, 115, 34, 41, 13, 10
            ]
        );
        Ok(())
    }

    pub fn init_logging() {
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .init();
    }

    #[tokio::test]
    async fn test_asset_read_binary_remote() -> anyhow::Result<()> {
        init_logging();
        // Create files
        let tc = Tome {
            eldritch: r#"print(assets.read_binary("remote_asset/just_a_remote_asset.txt"))"#
                .to_owned(),
            parameters: HashMap::new(),
            file_names: Vec::from(["remote_asset/just_a_remote_asset.txt".to_string()]),
        };

        // Run Eldritch (in it's own thread)
        let mut runtime = crate::start(123, tc.clone(), pb::config::Config::default_with_imix_verison("0.0.0")).await;

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
            assert!(msg.name == "remote_asset/just_a_remote_asset.txt");

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

        let mut found = false;
        for msg in runtime.messages() {
            if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                log::debug!("{}", m.text);
                assert_eq!(123, m.id);
                assert_eq!(
                    "[99, 104, 117, 110, 107, 49, 10, 99, 104, 117, 110, 107, 50, 10]\n"
                        .to_string(),
                    m.text
                );
                found = true;
            }
        }
        assert!(found);

        // Lastly, assert the file was written correctly

        Ok(())
    }
}
