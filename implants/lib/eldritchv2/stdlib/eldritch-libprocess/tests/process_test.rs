use eldritch_core::{Interpreter, Value};
use eldritch_libprocess::std::StdProcessLibrary;

#[test]
fn test_process_list() {
    let lib = StdProcessLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let res = interp.interpret("process.list()").unwrap();
    if let Value::List(l) = res {
        let list = l.read();
        assert!(!list.is_empty(), "Process list should not be empty");
        // Check first element structure
        if let Value::Dictionary(d) = &list[0] {
            let dict = d.read();
            assert!(dict.contains_key(&Value::String("pid".to_string())));
            assert!(dict.contains_key(&Value::String("name".to_string())));
        } else {
            panic!("Expected dict in list, got {:?}", list[0]);
        }
    } else {
        panic!("Expected list, got {:?}", res);
    }
}

#[test]
fn test_process_name() {
    let lib = StdProcessLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // Get pid of current process
    let pid = std::process::id() as i64;

    let code = format!("process.name({})", pid);
    let res = interp.interpret(&code).unwrap();

    if let Value::String(s) = res {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string, got {:?}", res);
    }
}

#[test]
fn test_process_info() {
    let lib = StdProcessLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // Get pid of current process
    let pid = std::process::id() as i64;

    let code = format!("process.info({})", pid);
    let res = interp.interpret(&code).unwrap();

    if let Value::Dictionary(d) = res {
        let dict = d.read();
        assert!(dict.contains_key(&Value::String("pid".to_string())));
        assert!(dict.contains_key(&Value::String("name".to_string())));
    } else {
        panic!("Expected dict, got {:?}", res);
    }
}

#[test]
fn test_process_netstat() {
    let lib = StdProcessLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    let res = interp.interpret("process.netstat()").unwrap();

    // Note: netstat might fail or return empty depending on permissions/environment.
    // We just check type if OK.
    if let Value::List(_) = res {
        // ok
    } else {
        panic!("Expected list, got {:?}", res);
    }
}
