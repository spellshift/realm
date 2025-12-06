#[cfg(feature = "fake_bindings")]
#[cfg(test)]
mod tests {
    use eldritch_core::Value;
    use eldritchv2::Interpreter;
    use eldritchv2::agent::fake::AgentFake;
    use std::sync::Arc;

    // Helper to create a fully loaded interpreter using the facade
    fn create_interp() -> Interpreter {
        let agent_mock = Arc::new(AgentFake::default());
        let task_id = 123;

        Interpreter::new()
            .with_default_libs()
            .with_task_context(agent_mock, task_id, vec![])
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
                "mkdir",
                "move",
                "parent_dir",
                "read",
                "read_binary",
                "remove",
                "replace",
                "replace_all",
                "temp_file",
                "template",
                "timestomp",
                "write",
            ],
        );
    }

    #[test]
    fn test_process_bindings() {
        check_bindings("process", &["info", "kill", "list", "name", "netstat"]);
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
                "shell",
                "write_reg_hex",
                "write_reg_int",
                "write_reg_str",
            ],
        );
    }

    #[test]
    fn test_pivot_bindings() {
        check_bindings(
            "pivot",
            &[
                "arp_scan",
                "bind_proxy",
                "ncat",
                "port_forward",
                "port_scan",
                "reverse_shell_pty",
                "reverse_shell_repl",
                "smb_exec",
                "ssh_copy",
                "ssh_exec",
            ],
        );
    }

    #[test]
    fn test_assets_bindings() {
        check_bindings("assets", &["copy", "list", "read", "read_binary"]);
    }

    #[test]
    fn test_crypto_bindings() {
        check_bindings(
            "crypto",
            &["aes_decrypt", "aes_encrypt", "md5", "sha1", "sha256"],
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
            &["file", "process_list", "ssh_key", "user_password"],
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
                "add_transport",
                "claim_tasks",
                "fetch_asset",
                "get_callback_interval",
                "get_config",
                "get_id",
                "get_platform",
                "get_transport",
                "kill",
                "list_tasks",
                "list_transports",
                "report_credential",
                "report_file",
                "report_process_list",
                "report_task_output",
                "reverse_shell",
                "set_callback_interval",
                "set_config",
                "set_transport",
                "sleep",
                "stop_task",
            ],
        );
    }
}
