use crate::runtime::{
    messages::{Message, ReportCredential},
    Environment,
};
use anyhow::Result;
use pb::{
    c2::ReportCredentialRequest,
    eldritch::{credential::Kind, Credential},
};
use starlark::eval::Evaluator;

pub fn ssh_key(starlark_eval: &Evaluator<'_, '_>, username: String, key: String) -> Result<()> {
    let env = Environment::from_extra(starlark_eval.extra)?;
    env.send(Message::from(ReportCredential {
        req: ReportCredentialRequest {
            task_id: env.id(),
            credential: Some(Credential {
                principal: username,
                secret: key,
                kind: Kind::SshKey.into(),
            }),
        },
    }))?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use anyhow::Error;
    use pb::{
        c2::ReportCredentialResponse,
        eldritch::{credential::Kind, Credential, Tome},
    };

    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                let mut runtime = crate::start(tc.id, tc.tome).await;
                runtime.finish().await;

                // TODO
                // runtime.collect_and_dispatch(mock).await;
            }
        )*
        }
    }

    struct TestCase {
        pub id: i64,
        pub tome: Tome,
        pub want_output: String,
        pub want_error: Option<Error>,
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
                want_output: String::from(""),
                want_error: None,
            },
    }
}
