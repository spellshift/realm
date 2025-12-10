use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn parent_dir(path: String) -> Result<String> {
    let mut res = PathBuf::from(&path);
    res.pop();
    Ok(res
        .to_str()
        .context("Failed to convert to str")?
        .to_string())
}

#[cfg(test)]
mod test {
    use crate::runtime::{messages::AsyncMessage, Message};
    use pb::eldritch::Tome;
    use std::collections::HashMap;

    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                // Run Eldritch (until finished)
                let mut runtime = crate::start(tc.id, tc.tome).await;
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
        simple_ssh: TestCase{
            id: 123,
            tome: Tome{
                eldritch: String::from(r#"print(file.parent_dir(input_params['path']))"#),
                #[cfg(not(target_os="windows"))]
                parameters: HashMap::from([(String::from("path"),String::from("/etc/ssh/sshd_config"))]),
                #[cfg(target_os="windows")]
                parameters: HashMap::from([(String::from("path"),String::from("C:\\ProgramData\\ssh\\sshd_config"))]),
                file_names: Vec::new(),
            },
            #[cfg(not(target_os="windows"))]
            want_text: String::from("/etc/ssh\n"),
            #[cfg(target_os="windows")]
            want_text: String::from("C:\\ProgramData\\ssh\n"),
        },
    }
}
