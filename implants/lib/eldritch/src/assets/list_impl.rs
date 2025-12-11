use anyhow::{Context, Result};
use starlark::{eval::Evaluator, values::list::ListRef};

pub fn list(starlark_eval: &Evaluator<'_, '_>) -> Result<Vec<String>> {
    let mut res: Vec<String> = Vec::new();
    let remote_assets = starlark_eval.module().get("remote_assets");

    if let Some(assets) = remote_assets {
        let tmp_list = ListRef::from_value(assets).context("`remote_assets` is not type list")?;
        for asset_path in tmp_list.iter() {
            let mut asset_path_string = asset_path.to_str();
            if let Some(local_asset_path_string) = asset_path_string.strip_prefix('"') {
                asset_path_string = local_asset_path_string.to_string();
            }
            if let Some(local_asset_path_string) = asset_path_string.strip_suffix('"') {
                asset_path_string = local_asset_path_string.to_string();
            }
            res.push(asset_path_string)
        }
        if !res.is_empty() {
            return Ok(res);
        }
    }

    for asset_path in super::Asset::iter() {
        res.push(asset_path.to_string());
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::runtime::{messages::AsyncMessage, Message};
    use pb::eldritch::Tome;

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
            test_asset_list_remote: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"print(assets.list())"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_text: String::from("[\"exec_script/hello_world.bat\", \"exec_script/hello_world.sh\", \"exec_script/main.eldritch\", \"exec_script/metadata.yml\", \"print/main.eldritch\", \"print/metadata.yml\"]\n"),
            },
            test_asset_list_local: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"print(assets.list())"#),
                    parameters: HashMap::new(),
                    file_names: Vec::from(["remote_asset/just_a_remote_asset.txt".to_string()]),
                },
                want_text: String::from("[\"remote_asset/just_a_remote_asset.txt\"]\n"),
            },
    }
}
