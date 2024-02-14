use crate::{pb::credential::Kind, pb::Credential, runtime::Environment};
use anyhow::Result;
use starlark::eval::Evaluator;

pub fn user_password(
    starlark_eval: &Evaluator<'_, '_>,
    username: String,
    password: String,
) -> Result<()> {
    let env = Environment::from_extra(starlark_eval.extra)?;
    env.report_credential(Credential {
        principal: username,
        secret: password,
        kind: Kind::Password.into(),
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
                    eldritch: String::from(r#"report.user_password(username="root", password="changeme")"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_credential: Credential {
                    principal: String::from("root"),
                    secret:  String::from("changeme"),
                    kind: Kind::Password.into(),
                },
                want_output: String::from(""),
                want_error: None,
            },
    }
}
