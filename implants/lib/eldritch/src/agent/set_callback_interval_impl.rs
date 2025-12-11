use crate::runtime::{messages::SetCallbackIntervalMessage, messages::SyncMessage, Environment};
use anyhow::Result;

pub fn set_callback_interval(env: &Environment, new_interval: u64) -> Result<()> {
    env.send(SyncMessage::from(SetCallbackIntervalMessage {
        id: env.id(),
        new_interval,
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
                let mut runtime = crate::start(tc.id, tc.tome, pb::config::Config::default_with_imix_verison("0.0.0")).await;
                runtime.finish().await;

                // Read Messages
                let mut found = false;
                for msg in runtime.messages() {
                    if let Message::Sync(SyncMessage::SetCallbackInterval(m)) = msg {
                        assert_eq!(tc.new_interval, m.new_interval);
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
        pub new_interval: u64,
    }

    test_cases! {
            change_interval: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"agent.set_callback_interval(10)"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                new_interval: 10,
            },
    }
}
