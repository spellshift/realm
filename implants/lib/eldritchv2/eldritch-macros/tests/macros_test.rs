mod tests {
    extern crate alloc;
    use eldritch_core::{Interpreter, Value};
    use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    // --- Library 1: File System ---

    #[eldritch_library("file")]
    pub trait LibFS {
        #[eldritch_method]
        fn move_file(&self, src: &str, dst: &str) -> Result<(), String>;

        #[eldritch_method("write_str")]
        fn write_string(&self, dst: &str, content: &str) -> Result<i64, String>;

        #[eldritch_method]
        fn complex_op(&self, ids: Vec<i64>, meta: BTreeMap<String, bool>) -> Result<bool, String>;

        #[eldritch_method]
        fn btree_ret(&self) -> Result<BTreeMap<String, i64>, String>;
    }

    // Mock implementation for LibFS
    #[derive(Debug, Default, Clone)]
    #[eldritch_library_impl(LibFS)]
    struct MockFS {
        ops: Arc<Mutex<Vec<String>>>,
    }

    impl LibFS for MockFS {
        fn move_file(&self, src: &str, dst: &str) -> Result<(), String> {
            self.ops
                .lock()
                .unwrap()
                .push(format!("move {src} -> {dst}"));
            Ok(())
        }

        fn write_string(&self, dst: &str, content: &str) -> Result<i64, String> {
            self.ops
                .lock()
                .unwrap()
                .push(format!("write {} len={}", dst, content.len()));
            Ok(content.len() as i64)
        }

        fn complex_op(&self, ids: Vec<i64>, meta: BTreeMap<String, bool>) -> Result<bool, String> {
            self.ops
                .lock()
                .unwrap()
                .push(format!("complex ids={ids:?} meta={meta:?}"));
            Ok(true)
        }

        fn btree_ret(&self) -> Result<BTreeMap<String, i64>, String> {
            let mut m = BTreeMap::new();
            m.insert("one".to_string(), 1);
            m.insert("two".to_string(), 2);
            Ok(m)
        }
    }

    // Second implementation for LibFS: DiskFS (simulated)
    #[derive(Debug, Default, Clone)]
    #[eldritch_library_impl(LibFS)]
    struct DiskFS {
        files: Arc<Mutex<BTreeMap<String, String>>>,
    }

    impl LibFS for DiskFS {
        fn move_file(&self, src: &str, dst: &str) -> Result<(), String> {
            let mut files = self.files.lock().unwrap();
            if let Some(content) = files.remove(src) {
                files.insert(dst.to_string(), content);
                Ok(())
            } else {
                Err("File not found".to_string())
            }
        }

        fn write_string(&self, dst: &str, content: &str) -> Result<i64, String> {
            let mut files = self.files.lock().unwrap();
            files.insert(dst.to_string(), content.to_string());
            Ok(content.len() as i64)
        }

        fn complex_op(
            &self,
            _ids: Vec<i64>,
            _meta: BTreeMap<String, bool>,
        ) -> Result<bool, String> {
            Ok(false)
        }

        fn btree_ret(&self) -> Result<BTreeMap<String, i64>, String> {
            Ok(BTreeMap::new())
        }
    }

    // --- Library 2: Network ---

    #[eldritch_library("net")]
    pub trait LibNet {
        #[eldritch_method]
        fn connect(&self, host: &str, port: i64) -> Result<bool, String>;

        #[eldritch_method]
        fn send(&self, data: &str) -> Result<i64, String>;
    }

    #[derive(Debug, Default, Clone)]
    #[eldritch_library_impl(LibNet)]
    struct MockNet {
        log: Arc<Mutex<Vec<String>>>,
    }

    impl LibNet for MockNet {
        fn connect(&self, host: &str, port: i64) -> Result<bool, String> {
            self.log
                .lock()
                .unwrap()
                .push(format!("connect {host}:{port}"));
            Ok(true)
        }

        fn send(&self, data: &str) -> Result<i64, String> {
            self.log.lock().unwrap().push(format!("send {data}"));
            Ok(data.len() as i64)
        }
    }

    #[test]
    fn test_macros_integration() {
        // --- Test 1: MockFS ---
        {
            let ops = Arc::new(Mutex::new(Vec::new()));
            let mock = MockFS { ops: ops.clone() };

            let mut interp = Interpreter::new();
            interp.register_lib(mock);

            let code = "file.move_file(\"/src\", dst=\"/dst\")\nfile.write_str(\"/foo\", content=\"hello\")";
            interp.interpret(code).unwrap();

            let ops = ops.lock().unwrap();
            assert_eq!(ops.len(), 2);
            assert_eq!(ops[0], "move /src -> /dst");
            assert_eq!(ops[1], "write /foo len=5");
        }

        // --- Test 2: DiskFS (Overwrites file lib) ---
        {
            let files = Arc::new(Mutex::new(BTreeMap::new()));
            let fs = DiskFS {
                files: files.clone(),
            };

            let mut interp = Interpreter::new();
            // Register 'file' library
            interp.register_lib(fs);

            let code =
                "file.write_str(\"/a.txt\", \"data\")\nfile.move_file(\"/a.txt\", \"/b.txt\")";

            interp.interpret(code).unwrap();

            let files = files.lock().unwrap();
            assert!(!files.contains_key("/a.txt"));
            assert_eq!(files.get("/b.txt").map(|s| s.as_str()), Some("data"));
        }

        // --- Test 3: Multiple Libraries (MockFS + MockNet) ---
        {
            let fs_ops = Arc::new(Mutex::new(Vec::new()));
            let mock_fs = MockFS {
                ops: fs_ops.clone(),
            };

            let net_log = Arc::new(Mutex::new(Vec::new()));
            let mock_net = MockNet {
                log: net_log.clone(),
            };

            let mut interp = Interpreter::new();
            // Re-register 'file' to be MockFS
            interp.register_lib(mock_fs);
            interp.register_lib(mock_net);

            let code = "file.move_file(\"/local\", \"/remote\")\nnet.connect(\"example.com\", 80)\nnet.send(\"GET / HTTP/1.1\")";

            interp.interpret(code).unwrap();

            {
                let fs = fs_ops.lock().unwrap();
                assert_eq!(fs[0], "move /local -> /remote");
            }
            {
                let net = net_log.lock().unwrap();
                assert_eq!(net[0], "connect example.com:80");
                assert_eq!(net[1], "send GET / HTTP/1.1");
            }
        }

        // --- Test 4: Complex Types (using MockFS) ---
        {
            // But we should create a fresh ops vector to verify output cleanly
            let ops = Arc::new(Mutex::new(Vec::new()));
            let mock = MockFS { ops: ops.clone() };

            let mut interp = Interpreter::new();
            interp.register_lib(mock);

            let code = "file.complex_op([1, 2, 3], {\"active\": True, \"hidden\": False})";
            let res = interp.interpret(code).unwrap();

            if let Value::Bool(b) = res {
                assert!(b);
            } else {
                panic!("Expected Bool result, got {res:?}");
            }

            {
                let ops = ops.lock().unwrap();
                let last_op = ops.last().unwrap();
                assert!(last_op.contains("complex ids=[1, 2, 3]"));
                assert!(last_op.contains("meta={"));
            }
        }

        // --- Test 5: Return BTreeMap ---
        {
            let ops = Arc::new(Mutex::new(Vec::new()));
            let mock = MockFS { ops: ops.clone() };

            let mut interp = Interpreter::new();
            interp.register_lib(mock);

            let code = "file.btree_ret()";
            let res = interp.interpret(code).unwrap();

            if let Value::Dictionary(d) = res {
                let dict = d.read();
                assert_eq!(dict.get(&Value::String("one".to_string())).unwrap(), &Value::Int(1));
                assert_eq!(dict.get(&Value::String("two".to_string())).unwrap(), &Value::Int(2));
            } else {
                panic!("Expected Dictionary, got {res:?}");
            }
        }

        // --- Test 6: Introspection ---
        {
            let ops = Arc::new(Mutex::new(Vec::new()));
            let mock = MockFS { ops: ops.clone() };

            // MockFS is registered
            let mut interp = Interpreter::new();
            interp.register_lib(mock);

            let code = "dir(file)";
            let res = interp.interpret(code).unwrap();

            if let Value::List(l) = res {
                let list = l.read();
                let mut strings: Vec<String> = list.iter().map(|v| v.to_string()).collect();
                strings.sort();
                assert!(strings.contains(&"move_file".to_string()));
                assert!(strings.contains(&"write_str".to_string()));
                assert!(strings.contains(&"btree_ret".to_string()));
            } else {
                panic!("Expected List, got {res:?}");
            }
        }
    }
}
