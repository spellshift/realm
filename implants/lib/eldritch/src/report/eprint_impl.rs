use crate::runtime::{messages::ReportErrorMessage, Environment};
use anyhow::Result;

pub fn eprint(env: &Environment, message: String) -> Result<()> {
    env.send(ReportErrorMessage {
        id: env.id(),
        error: message,
    });
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::runtime::Message;
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
                    if let Message::ReportError(m) = msg {
                        assert_eq!(tc.id, m.id);
                        assert_eq!(tc.want_error, m.error);
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
        pub want_error: String,
    }

    test_cases! {
            one_error: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"report.eprint(message="Beep Boop an error occured")"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_error: String::from("Beep Boop an error occured"),
            },
    }
}
