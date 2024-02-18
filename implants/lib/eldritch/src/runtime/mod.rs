mod drain;
mod environment;
mod eprint_impl;
mod eval;
pub mod messages;

pub(crate) use environment::Environment;
pub use eval::{expression, start, Runtime};
pub use messages::Message;

#[cfg(test)]
mod tests {
    use crate::runtime::Message;
    use pb::eldritch::Tome;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    macro_rules! runtime_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                let mut runtime = crate::start(tc.id, tc.tome).await;
                runtime.finish().await;

                let mut text = Vec::new();
                for msg in runtime.messages() {
                    match msg {
                        Message::ReportText(m) => text.push(m.text),
                        Message::ReportError(m) => assert_eq!(tc.want_error, Some(m.error)),
                        _ => {},
                    };
                }

                assert_eq!(tc.want_text, text.join(""));
            }
        )*
        }
    }

    struct TestCase {
        pub id: i64,
        pub tome: Tome,
        pub want_text: String,
        pub want_error: Option<String>,
    }

    runtime_tests! {
        simple_run: TestCase{
            id: 123,
            tome: Tome{
                eldritch: String::from("print(1+1)"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from("2"),
            want_error: None,
        },
        multi_print: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from(r#"print("oceans "); print("rise, "); print("empires "); print("fall")"#),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"oceans rise, empires fall"#),
            want_error: None,
        },
        input_params: TestCase{
            id: 123,
            tome: Tome {
                            eldritch: r#"print(input_params['cmd2'])"#.to_string(),
                            parameters: HashMap::from([
                                ("cmd".to_string(), "id".to_string()),
                                ("cmd2".to_string(), "echo hello_world".to_string()),
                                ("cmd3".to_string(), "ls -lah /tmp/".to_string()),
                            ]),
                            file_names: Vec::new(),
                        },
                        want_text: String::from("echo hello_world"),
                        want_error: None,
        },
        file_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(file))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["append", "compress", "copy", "download", "exists", "find", "follow", "is_dir", "is_file", "list", "mkdir", "moveto", "read", "remove", "replace", "replace_all", "template", "timestomp", "write"]"#),
            want_error: None,
        },
        process_bindings: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from("print(dir(process))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["info", "kill", "list", "name", "netstat"]"#),
            want_error: None,
        },
        sys_bindings: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from("print(dir(sys))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["dll_inject", "dll_reflect", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_reg", "get_user", "hostname", "is_linux", "is_macos", "is_windows", "shell", "write_reg_hex", "write_reg_int", "write_reg_str"]"#),
            want_error: None,
        },
        pivot_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(pivot))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_copy", "ssh_exec", "ssh_password_spray"]"#),
            want_error: None,
        },
        assets_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(assets))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["copy", "list", "read", "read_binary"]"#),
            want_error: None,
        },
        crypto_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(crypto))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "to_json"]"#),
            want_error: None,
        },
        time_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(time))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["format_to_epoch", "format_to_readable", "now", "sleep"]"#),
            want_error: None,
        },
        report_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(report))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["file", "process_list", "ssh_key", "user_password"]"#),
            want_error: None,
        },
        regex_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(regex))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from(r#"["match", "match_all", "replace", "replace_all"]"#),
            want_error: None,
        },
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 128)]
    async fn test_library_async() -> anyhow::Result<()> {
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap())
            .clone()
            .replace('\\', "\\\\");
        let eldritch =
            format!(r#"file.download("https://www.google.com/", "{path}"); print("ok")"#);
        let mut runtime = crate::start(
            123,
            Tome {
                eldritch,
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
        )
        .await;
        runtime.finish().await;

        // TODO: Stuff
        // let out = runtime.collect_text();
        // let err = runtime.collect_errors().pop();
        // assert!(err.is_none(), "failed with err {}", err.unwrap());
        // assert!(tmp_file.as_file().metadata().unwrap().len() > 5);
        // assert_eq!("ok", out.join(""));
        Ok(())
    }
}
