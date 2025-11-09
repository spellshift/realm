use crate::runtime::{
    messages::{AsyncMessage, ReportFileMessage},
    Environment,
};
use anyhow::Result;

pub fn file(env: &Environment, path: String) -> Result<()> {
    env.send(AsyncMessage::from(ReportFileMessage { id: env.id(), path }))?;
    Ok(())
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
                    if let Message::Async(AsyncMessage::ReportFile(m)) = msg {
                        assert_eq!(tc.id, m.id);
                        assert_eq!(tc.want_path, m.path);
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
        pub want_path: String,
    }

    test_cases! {
            one_file: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"report.file(path="/etc/passwd")"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_path: String::from("/etc/passwd"),
            },
    }
}
