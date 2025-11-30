#[cfg(feature = "std")]
mod tests {
    extern crate alloc;
    use eldritchv2::{eldritch_interface, eldritch_library, eldritch_method, register_lib, Interpreter, Value};
    use std::sync::{Arc, Mutex};

    // Define the trait with the interface macro
    #[eldritch_interface]
    pub trait RuntimeFS {
        #[eldritch_method]
        fn move_file(&self, src: &str, dst: &str) -> Result<(), String>;

        #[eldritch_method("write_str")]
        fn write_string(&self, dst: &str, content: &str) -> Result<i64, String>;
    }

    // Mock implementation
    #[derive(Debug, Default, Clone)]
    struct MockFS {
        ops: Arc<Mutex<Vec<String>>>,
    }

    impl RuntimeFS for MockFS {
        fn move_file(&self, src: &str, dst: &str) -> Result<(), String> {
            self.ops.lock().unwrap().push(format!("move {} -> {}", src, dst));
            Ok(())
        }

        fn write_string(&self, dst: &str, content: &str) -> Result<i64, String> {
            self.ops.lock().unwrap().push(format!("write {} len={}", dst, content.len()));
            Ok(content.len() as i64)
        }
    }

    // Define the library struct
    #[derive(Debug)]
    #[eldritch_library("file")]
    struct LibFS<FS: RuntimeFS> {
        fs: FS,
    }

    #[test]
    fn test_macros_and_dispatch() {
        let ops = Arc::new(Mutex::new(Vec::new()));
        let mock = MockFS { ops: ops.clone() };
        let lib = LibFS { fs: mock };

        register_lib(lib);

        let mut interp = Interpreter::new();

        // Test move_file with keyword args
        // Using explicit string to avoid indent issues
        let code1 = "file.move_file(\"/src\", dst=\"/dst\")";
        interp.interpret(code1).unwrap();

        {
            let ops = ops.lock().unwrap();
            assert_eq!(ops.len(), 1);
            assert_eq!(ops[0], "move /src -> /dst");
        }

        // Test write_str (renamed)
        let code2 = "file.write_str(\"/foo\", content=\"hello\")";
        let res = interp.interpret(code2).unwrap();

        {
            let ops = ops.lock().unwrap();
            assert_eq!(ops.len(), 2);
            assert_eq!(ops[1], "write /foo len=5");
        }

        if let Value::Int(i) = res {
            assert_eq!(i, 5);
        } else {
            panic!("Expected Int result, got {:?}", res);
        }
    }

    #[test]
    fn test_dir_introspection() {
        let ops = Arc::new(Mutex::new(Vec::new()));
        let mock = MockFS { ops: ops.clone() };
        let lib = LibFS { fs: mock };

        register_lib(lib);

        let mut interp = Interpreter::new();
        let code = "dir(file)";
        let res = interp.interpret(code).unwrap();

        if let Value::List(l) = res {
            let list = l.borrow();
            let mut strings: Vec<String> = list.iter().map(|v| v.to_string()).collect();
            strings.sort();
            assert!(strings.contains(&"move_file".to_string()));
            assert!(strings.contains(&"write_str".to_string()));
        } else {
            panic!("Expected List, got {:?}", res);
        }
    }
}
