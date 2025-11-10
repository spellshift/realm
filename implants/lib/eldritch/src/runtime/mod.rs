mod drain;
mod environment;
pub mod eprint_impl;
mod eval;
pub mod messages;

pub(crate) use environment::Environment;
pub use eval::{start, Runtime};
pub use messages::Message;

#[cfg(test)]
mod tests {
    use crate::runtime::{messages::AsyncMessage, Message};
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
                        Message::Async(am) => {
                            match am {
                                AsyncMessage::ReportText(m) => text.push(m.text),
                                AsyncMessage::ReportError(m) => assert_eq!(tc.want_error, Some(m.error)),
                                _ => {},
                            }
                        },
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
            want_text: format!("{}\n", "2"),
            want_error: None,
        },
        multi_print: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from(r#"print("oceans "); print("rise, "); print("empires "); print("fall")"#),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: String::from("oceans \nrise, \nempires \nfall\n"),
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
                        want_text: format!("{}\n", "echo hello_world"),
                        want_error: None,
        },
        file_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(file))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["append", "compress", "copy", "decompress", "exists", "find", "follow", "is_dir", "is_file", "list", "mkdir", "moveto", "parent_dir", "read", "read_binary", "remove", "replace", "replace_all", "temp_file", "template", "timestomp", "write"]"#),
            want_error: None,
        },
        process_bindings: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from("print(dir(process))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["info", "kill", "list", "name", "netstat"]"#),
            want_error: None,
        },
        sys_bindings: TestCase {
            id: 123,
            tome: Tome{
                eldritch: String::from("print(dir(sys))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["dll_inject", "dll_reflect", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_reg", "get_user", "hostname", "is_bsd", "is_linux", "is_macos", "is_windows", "shell", "write_reg_hex", "write_reg_int", "write_reg_str"]"#),
            want_error: None,
        },
        pivot_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(pivot))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "reverse_shell_pty", "smb_exec", "ssh_copy", "ssh_exec"]"#),
            want_error: None,
        },
        assets_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(assets))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["copy", "list", "read", "read_binary"]"#),
            want_error: None,
        },
        crypto_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(crypto))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "is_json", "to_json"]"#),
            want_error: None,
        },
        time_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(time))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["format_to_epoch", "format_to_readable", "now", "sleep"]"#),
            want_error: None,
        },
        random_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(random))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["bool", "int", "string"]"#),
            want_error: None,
        },
        report_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(report))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["file", "process_list", "ssh_key", "user_password"]"#),
            want_error: None,
        },
        regex_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(regex))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["match", "match_all", "replace", "replace_all"]"#),
            want_error: None,
        },
        http_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(http))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["download", "get", "post"]"#),
            want_error: None,
        },
        agent_bindings: TestCase {
            id: 123,
            tome: Tome {
                eldritch: String::from("print(dir(agent))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_text: format!("{}\n", r#"["eval", "set_callback_interval", "set_callback_uri"]"#),
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
            format!(r#"http.download("https://www.google.com/", "{path}"); print("ok")"#);
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
