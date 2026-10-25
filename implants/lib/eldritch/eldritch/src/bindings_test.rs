use crate::Interpreter;
#[cfg(feature = "stdlib")]
use crate::agent::fake::AgentFake;
use eldritch_core::Value;
use std::sync::Arc;

// Helper to create a fully loaded interpreter using the facade
fn create_interp() -> Interpreter {
    #[cfg(feature = "stdlib")]
    {
        use eldritch_libassets::std::EmptyAssets;
        use pb::c2::TaskContext;

        let agent_mock = Arc::new(AgentFake);
        let task_context = TaskContext {
            task_id: 123,
            jwt: "a test jwt".to_string(),
        };
        let backend = Arc::new(EmptyAssets {});
        // with_default_libs registers Std then Fake (fake wins).
        // with_context re-registers Std (std wins – bug). Chain with_fake_agent
        // to make fake win again so tests use fake impls.
        Interpreter::new()
            .with_default_libs()
            .with_context(
                agent_mock,
                eldritch_agent::Context::Task(task_context),
                vec![],
                backend,
            )
            .with_fake_agent()
    }
    #[cfg(not(feature = "stdlib"))]
    {
        Interpreter::new().with_fake_agent()
    }
}

fn check_bindings(module: &str, expected: &[&str]) {
    let mut interp = create_interp();
    let code = format!("dir({module})");
    let val = interp.interpret(&code).unwrap();

    if let Value::List(l) = val {
        let list = l.read();
        let mut actual: Vec<String> = list
            .iter()
            .map(|v| v.to_string().replace("\"", ""))
            .collect();
        actual.sort();

        let mut expected_sorted: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        expected_sorted.sort();

        assert_eq!(actual, expected_sorted, "Mismatch for module {module}");
    } else {
        panic!("Expected list for dir({module})");
    }
}

#[test]
fn test_file_bindings() {
    check_bindings(
        "file",
        &[
            "append",
            "compress",
            "copy",
            "decompress",
            "exists",
            "find",
            "follow",
            "is_dir",
            "is_file",
            "list",
            "list_named_pipes",
            "list_recent",
            "mkdir",
            "move",
            "parent_dir",
            "pwd",
            "read",
            "read_binary",
            "read_named_pipe",
            "remove",
            "replace",
            "replace_all",
            "temp_file",
            "template",
            "template_str",
            "timestomp",
            "tmp_dir",
            "write",
            "write_binary",
        ],
    );
}

#[test]
fn test_process_bindings() {
    check_bindings(
        "process",
        &["info", "kill", "list", "name", "netstat", "signal"],
    );
}

#[test]
fn test_sys_bindings() {
    check_bindings(
        "sys",
        &[
            "dll_inject",
            "dll_reflect",
            "exec",
            "get_env",
            "get_ip",
            "get_os",
            "get_pid",
            "get_reg",
            "get_user",
            "hostname",
            "is_bsd",
            "is_linux",
            "is_macos",
            "is_windows",
            "list_users",
            "shell",
            "write_reg",
        ],
    );
}

#[test]
fn test_pivot_bindings() {
    check_bindings(
        "pivot",
        &[
            "arp_scan",
            "create_portal",
            // "bind_proxy", // Not implemented in Fake
            "ncat",
            // "port_forward", // Not implemented in Fake
            "port_scan",
            // "smb_exec", // Not implemented in Fake
            "ssh_copy",
            "ssh_deploy",
            "ssh_exec",
            "ssh_session",
        ],
    );
}

#[test]
fn test_ssh_session_object_methods() {
    // with_fake_agent returns FakeSshSessionHandle which has exec + close
    let mut interp = create_interp();
    let code = r#"
s = pivot.ssh_session("127.0.0.1", 22, "root", "pass", None, None, None)
dir(s)
"#;
    let val = interp
        .interpret(code)
        .expect("interpret ssh_session failed");
    if let Value::List(l) = val {
        let list = l.read();
        let actual: Vec<String> = list
            .iter()
            .map(|v| v.to_string().replace("\"", ""))
            .collect();
        assert!(
            actual.contains(&"exec".to_string()),
            "ssh_session dir should contain exec, got {actual:?}"
        );
        assert!(
            actual.contains(&"close".to_string()),
            "ssh_session dir should contain close, got {actual:?}"
        );
    } else {
        panic!("Expected list for dir(ssh_session)");
    }
}

#[test]
fn test_ssh_session_fake_exec_and_close() {
    let mut interp = create_interp();
    // Fake exec returns stdout="fake output"
    let code = r#"
s = pivot.ssh_session("127.0.0.1", 22, "root", "pass", None, None, None)
r = s.exec("whoami")
r["stdout"]
"#;
    let val = interp.interpret(code).expect("interpret exec failed");
    assert_eq!(val.to_string(), "fake output");

    // After close, exec should raise.
    let code2 = r#"
s = pivot.ssh_session("127.0.0.1", 22, "root", "pass", None, None, None)
s.close()
s.exec("whoami")
"#;
    let res2 = interp.interpret(code2);
    assert!(
        res2.is_err(),
        "expected error after close, got ok: {res2:?}"
    );
}

#[test]
fn test_ssh_session_fake_reuse_multiple_execs() {
    let mut interp = create_interp();
    let code = r#"
s = pivot.ssh_session("127.0.0.1", 22, "root", "pass", None, None, None)
r1 = s.exec("whoami")
r2 = s.exec("id")
r3 = s.exec("hostname")
s.close()
r1["stdout"] + "|" + r2["stdout"] + "|" + r3["stdout"]
"#;
    let val = interp.interpret(code).expect("multi exec should succeed");
    assert_eq!(val.to_string(), "fake output|fake output|fake output");
}

#[test]
fn test_assets_bindings() {
    check_bindings("assets", &["copy", "list", "read", "read_binary"]);
}

#[test]
fn test_crypto_bindings() {
    check_bindings(
        "crypto",
        &[
            "aes_decrypt",
            "aes_decrypt_file",
            "aes_encrypt",
            "aes_encrypt_file",
            "decode_b64",
            "decode_utf16le",
            "encode_b64",
            "encode_utf16le",
            "from_json",
            "hash_file",
            "is_json",
            "md5",
            "sha1",
            "sha256",
            "to_json",
        ],
    );
}

#[test]
fn test_time_bindings() {
    check_bindings(
        "time",
        &["format_to_epoch", "format_to_readable", "now", "sleep"],
    );
}

#[test]
fn test_random_bindings() {
    check_bindings("random", &["bool", "bytes", "int", "string", "uuid"]);
}

#[test]
fn test_report_bindings() {
    check_bindings(
        "report",
        &[
            "file",
            "ntlm_hash",
            "process_list",
            "screenshot",
            "ssh_key",
            "user_password",
        ],
    );
}

#[test]
fn test_regex_bindings() {
    check_bindings("regex", &["match", "match_all", "replace", "replace_all"]);
}

#[test]
fn test_http_bindings() {
    check_bindings("http", &["download", "get", "post"]);
}

#[test]
fn test_agent_bindings() {
    check_bindings(
        "agent",
        &[
            "_terminate_this_process_clowntown",
            "get_callback_interval",
            "get_config",
            "get_transport",
            "list_tasks",
            "list_transports",
            "reset_transport",
            "set_callback_interval",
            "set_callback_uri",
            "stop_task",
        ],
    );
}
