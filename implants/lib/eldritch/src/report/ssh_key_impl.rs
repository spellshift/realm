use crate::runtime::{
    messages::{AsyncMessage, ReportCredentialMessage},
    Environment,
};
use anyhow::Result;
use pb::eldritch::{credential::Kind, Credential};

pub fn ssh_key(env: &Environment, username: String, key: String) -> Result<()> {
    env.send(AsyncMessage::from(ReportCredentialMessage {
        id: env.id(),
        credential: Credential {
            principal: username,
            secret: key,
            kind: Kind::SshKey.into(),
        },
    }))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::runtime::{messages::AsyncMessage, Message};
    use pb::eldritch::{credential::Kind, Credential, Tome};
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
                    if let Message::Async(AsyncMessage::ReportCredential(m)) = msg {
                        assert_eq!(tc.want_credential, m.credential);
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
        pub want_credential: Credential,
    }

    test_cases! {
            one_credential: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"report.ssh_key(username="root", key="---BEGIN---youknowtherest")"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_credential: Credential {
                    principal: String::from("root"),
                    secret:  String::from("---BEGIN---youknowtherest"),
                    kind: Kind::SshKey.into(),
                },
            },
    }
}
