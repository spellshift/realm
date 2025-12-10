use crate::runtime::{messages::SetCallbackUriMessage, messages::SyncMessage, Environment};
use anyhow::Result;

pub fn set_callback_uri(env: &Environment, new_uri: String) -> Result<()> {
    env.send(SyncMessage::from(SetCallbackUriMessage {
        id: env.id(),
        new_uri,
    }))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::runtime::{messages::SyncMessage, Message};
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
                    if let Message::Sync(SyncMessage::SetCallbackUri(m)) = msg {
                        assert_eq!(tc.new_uri, m.new_uri);
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
        pub new_uri: String,
    }

    test_cases! {
            change_interval: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"agent.set_callback_uri("https://127.0.0.1")"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                new_uri: String::from("https://127.0.0.1"),
            },
    }
}
