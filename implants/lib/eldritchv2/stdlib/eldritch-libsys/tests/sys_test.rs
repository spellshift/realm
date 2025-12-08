use eldritch_core::{Interpreter, Value};
use eldritch_libsys::std::StdSysLibrary;

#[test]
fn test_sys_bindings() {
    // StdSysLibrary is a unit struct but doesn't derive Default.
    // It's defined as `pub struct StdSysLibrary;`.
    // So we can use StdSysLibrary directly.
    let lib = StdSysLibrary;
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // sys.get_env()
    let res = interp.interpret("sys.get_env()").unwrap();
    if let Value::Dictionary(d) = res {
        // Environment vars
        assert!(!d.read().is_empty());
    } else {
        panic!("Expected dict, got {:?}", res);
    }

    // sys.get_pid()
    let res = interp.interpret("sys.get_pid()").unwrap();
    if let Value::Int(pid) = res {
        assert!(pid > 0);
    } else {
        panic!("Expected int, got {:?}", res);
    }

    // sys.hostname()
    let res = interp.interpret("sys.hostname()").unwrap();
    if let Value::String(s) = res {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string, got {:?}", res);
    }

    // sys.shell
    let cmd = if cfg!(windows) { "cmd /c echo hello" } else { "echo hello" };
    let code = format!("sys.shell('{}')", cmd);
    let res = interp.interpret(&code).unwrap();
    if let Value::Dictionary(d) = res {
        let dict = d.read();
        // check stdout
        let out = dict.get(&Value::String("stdout".to_string()));
        if let Some(Value::String(s)) = out {
            assert!(s.trim() == "hello");
        }
    } else {
        panic!("Expected dict, got {:?}", res);
    }

    // sys.get_os()
    let res = interp.interpret("sys.get_os()").unwrap();
    if let Value::Dictionary(_) = res {
        // ok
    } else {
        panic!("Expected dict, got {:?}", res);
    }

    // sys.get_user()
    let res = interp.interpret("sys.get_user()").unwrap();
    if let Value::Dictionary(_) = res {
        // ok
    } else {
        panic!("Expected dict, got {:?}", res);
    }

    // sys.get_ip()
    let res = interp.interpret("sys.get_ip()");
    assert!(res.is_ok());

    // sys.is_windows()
    let res = interp.interpret("sys.is_windows()").unwrap();
    if let Value::Bool(_) = res {
    } else {
        panic!("Expected bool");
    }
}
