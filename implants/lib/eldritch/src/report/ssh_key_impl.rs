use crate::{pb::credential::Kind, pb::Credential, runtime::Environment};
use anyhow::Result;
use starlark::eval::Evaluator;

pub fn ssh_key(starlark_eval: &Evaluator<'_, '_>, username: String, key: String) -> Result<()> {
    let env = Environment::from_extra(starlark_eval.extra)?;
    env.report_credential(Credential {
        principal: username,
        secret: key,
        kind: Kind::SshKey.into(),
    })?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::pb::{credential::Kind, Credential, Tome};
    use anyhow::Error;

    macro_rules! test_cases {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;
                let mut runtime = crate::start(tc.tome).await;
                runtime.finish().await;

                let want_err_str = match tc.want_error {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                let err_str = match runtime.collect_errors().pop() {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                assert_eq!(want_err_str, err_str);
                assert_eq!(tc.want_output, runtime.collect_text().join(""));
                assert_eq!(Some(tc.want_credential), runtime.collect_credentials().pop());
            }
        )*
        }
    }

    struct TestCase {
        pub tome: Tome,
        pub want_output: String,
        pub want_error: Option<Error>,
        pub want_credential: Credential,
    }

    test_cases! {
            one_credential: TestCase{
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
