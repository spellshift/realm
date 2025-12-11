use std::sync::mpsc::{channel, Receiver};

use anyhow::{Context, Result};
use pb::c2::FetchAssetResponse;
use starlark::{eval::Evaluator, values::list::ListRef};

use crate::runtime::{
    messages::{AsyncMessage, FetchAssetMessage},
    Environment,
};

fn read_local(src: String) -> Result<String> {
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

fn read_remote(rx: Receiver<FetchAssetResponse>) -> Result<String> {
    // Wait for our first chunk
    let resp = rx.recv()?;

    let mut res = String::new();

    res.push_str(&String::from_utf8_lossy(resp.chunk.as_slice()));

    // Listen for more chunks and write them
    for resp in rx {
        res.push_str(&String::from_utf8_lossy(resp.chunk.as_slice()));
    }

    Ok(res)
}

pub fn read(starlark_eval: &Evaluator<'_, '_>, src: String) -> Result<String> {
    let remote_assets = starlark_eval.module().get("remote_assets");

    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("`remote_assets` is not type list")?;
        let src_value = starlark_eval.module().heap().alloc_str(&src);

        if tmp_list.contains(&src_value.to_value()) {
            let env = Environment::from_extra(starlark_eval.extra)?;
            let (tx, rx) = channel();
            env.send(AsyncMessage::from(FetchAssetMessage { name: src, tx }))?;

            return read_remote(rx);
        }
    }
    read_local(src)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::runtime::{messages::AsyncMessage, messages::FetchAssetMessage, Message};
    use pb::{c2::FetchAssetResponse, eldritch::Tome};

    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                // Run Eldritch (until finished)
                let mut runtime = crate::start(tc.id, tc.tome, pb::config::Config::default_with_imix_verison("0.0.0")).await;

                runtime.finish().await;

                // Read Messages
                let mut found = false;
                for msg in runtime.messages() {
                    if let Message::Async(AsyncMessage::ReportText(m)) = msg {
                        println!("{}", m.text);
                        assert_eq!(tc.id, m.id);
                        assert_eq!(tc.want_text, m.text);
                        found = true;
                    }
                }
                assert!(found);
            }
        )*
        }
    }

    struct TestCase {
        pub id: i64,
        pub tome: Tome,
        pub want_text: String,
    }

    test_cases! {
        test_asset_read_local: TestCase{
            id: 123,
            tome: Tome{
                eldritch: String::from(r#"print(assets.read("print/main.eldritch").strip())"#),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from("print(\"This script just prints\")\n"),
        },

    }

    #[tokio::test]
    async fn test_asset_read_remote() -> anyhow::Result<()> {
        // Create files
        let tc = Tome {
            eldritch: r#"print(assets.read("remote_asset/just_a_remote_asset.txt").strip())"#
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
                assert_eq!(123, m.id);
                assert_eq!("chunk1\nchunk2\n", m.text);
                found = true;
            }
        }
        assert!(found);

        // Lastly, assert the file was written correctly

        Ok(())
    }
}
