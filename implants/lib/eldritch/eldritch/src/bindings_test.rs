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
        Interpreter::new().with_default_libs().with_context(
            agent_mock,
            eldritch_agent::Context::Task(task_context),
            vec![],
            backend,
        )
    }
    #[cfg(not(feature = "stdlib"))]
    {
        Interpreter::new()
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
            "read_named_pipe",
            "remove",
            "replace",
            "replace_all",
            "temp_file",
            "template",
            "template_str",
            "timestomp",
            "tmp",
            "tmp_dir",
            "which",
            "write",
        ],
    );
}

#[test]
fn test_process_bindings() {
    check_bindings(
        "process",
        &[
            "cwd", "env", "info", "kill", "list", "name", "netstat", "pid", "signal",
        ],
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
            "impersonate",
            "info",
            "is_bsd",
            "is_linux",
            "is_macos",
            "is_windows",
            "list_users",
            "shell",
            "tokens",
            "use_token",
            "whereis",
            "write_reg",
        ],
    );
}

#[test]
fn test_aes_bindings() {
    check_bindings(
        "aes",
        &["decrypt", "decrypt_file", "encrypt", "encrypt_file"],
    );
}

#[test]
fn test_base64_bindings() {
    check_bindings("base64", &["decode", "encode"]);
}

#[test]
fn test_gzip_bindings() {
    check_bindings(
        "gzip",
        &["compress", "compress_file", "decompress", "decompress_file"],
    );
}

#[test]
fn test_hash_bindings() {
    check_bindings("hash", &["file", "md5", "sha1", "sha256"]);
}

#[test]
fn test_json_bindings() {
    check_bindings("json", &["decode", "encode", "is_valid"]);
}

#[test]
fn test_linux_bindings() {
    check_bindings("linux", &["uname"]);
}

#[test]
fn test_net_bindings() {
    check_bindings("net", &["conns", "interfaces", "tcp", "udp"]);
}

#[test]
fn test_portal_bindings() {
    check_bindings("portal", &["create"]);
}

#[test]
fn test_scan_bindings() {
    check_bindings("scan", &["arp", "ports"]);
}

#[test]
fn test_ssh_bindings() {
    check_bindings("ssh", &["copy", "deploy", "exec"]);
}

#[test]
fn test_user_bindings() {
    check_bindings("user", &["info", "list"]);
}

#[test]
fn test_registry_bindings() {
    check_bindings("registry", &["read", "write"]);
}

#[test]
fn test_windows_bindings() {
    check_bindings(
        "windows",
        &[
            "dll_inject",
            "dll_reflect",
            "impersonate",
            "tokens",
            "use_token",
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
        ],
    );
}

#[test]
fn test_assets_bindings() {
    check_bindings("assets", &["copy", "list", "read"]);
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
        &[
            "to_epoch",
            "format",
            "format_to_epoch",
            "format_to_readable",
            "now",
            "sleep",
        ],
    );
}

#[test]
fn test_template_bindings() {
    check_bindings("template", &["render", "render_file"]);
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
            "callback_interval",
            "config",
            "transport",
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

#[test]
fn test_deprecated_delegation_and_warning() {
    use eldritch_core::BufferPrinter;
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = create_interp().with_printer(printer.clone());

    let res = interp.interpret("crypto.encode_b64('hello')").unwrap();
    let new_res = interp.interpret("base64.encode('hello')").unwrap();
    assert_eq!(res, new_res);
    let err_out = printer.read_err();
    assert!(
        err_out.contains("crypto.encode_b64 has been deprecated and will be removed in a future version of eldritch. Please use base64.encode instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    let os_old = interp.interpret("sys.get_os()").unwrap();
    let os_new = interp.interpret("sys.info()").unwrap();
    assert_eq!(os_old, os_new);
    let err_out = printer.read_err();
    assert!(
        err_out.contains("sys.get_os has been deprecated and will be removed in a future version of eldritch. Please use sys.info instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    let whereis_old = interp.interpret("sys.whereis('sh')").unwrap();
    let which_new = interp.interpret("file.which('sh')").unwrap();
    assert_eq!(whereis_old, which_new);
    let err_out = printer.read_err();
    assert!(
        err_out.contains("sys.whereis has been deprecated and will be removed in a future version of eldritch. Please use file.which instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    let _ = interp.interpret("sys.get_reg('HKCU\\\\Software')");
    let err_out = printer.read_err();
    assert!(
        err_out.contains("sys.get_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.read instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    let _ = interp.interpret("sys.write_reg('HKCU\\\\Software', 'Test', 'REG_SZ', 'val')");
    let err_out = printer.read_err();
    assert!(
        err_out.contains("sys.write_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.write instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    let _ = interp.interpret("pivot.ncat('127.0.0.1', 1, 'hi', 'tcp')");
    let err_out = printer.read_err();
    assert!(
        err_out.contains("pivot.ncat has been deprecated and will be removed in a future version of eldritch. Please use net.tcp & net.udp instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."),
        "Unexpected err_out: {err_out}"
    );

    // Verify new 0.4.1 functions work end-to-end through the interpreter
    let utf8_roundtrip = interp
        .interpret("'eldritch 0.4.1'.encode().decode()")
        .unwrap();
    assert_eq!(
        utf8_roundtrip,
        eldritch_core::Value::String("eldritch 0.4.1".into())
    );

    let utf16le_roundtrip = interp
        .interpret("'eldritch 0.4.1'.encode('utf16le').decode('utf16le')")
        .unwrap();
    assert_eq!(
        utf16le_roundtrip,
        eldritch_core::Value::String("eldritch 0.4.1".into())
    );

    let gzip_roundtrip = interp
        .interpret("gzip.decompress(gzip.compress('compress me'.encode())).decode()")
        .unwrap();
    assert_eq!(
        gzip_roundtrip,
        eldritch_core::Value::String("compress me".into())
    );

    let uname_res = interp.interpret("linux.uname()['sysname']").unwrap();
    assert_eq!(uname_res, eldritch_core::Value::String("Linux".into()));
}
