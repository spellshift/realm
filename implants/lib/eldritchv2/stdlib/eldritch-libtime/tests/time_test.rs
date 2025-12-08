use eldritch_core::{Interpreter, Value};
use eldritch_libtime::std::StdTimeLibrary;

#[test]
fn test_time_bindings() {
    let lib = StdTimeLibrary;
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // time.now()
    let res = interp.interpret("time.now()").unwrap();
    if let Value::Int(ts) = res {
        assert!(ts > 0);
    } else {
        panic!("Expected int, got {:?}", res);
    }

    // time.format_to_readable
    // args: epoch, format
    let epoch = 1600000000;
    let format = "%Y-%m-%d";
    let code = format!("time.format_to_readable({}, '{}')", epoch, format);
    let res = interp.interpret(&code).unwrap();
    if let Value::String(s) = res {
        assert_eq!(s, "2020-09-13");
    } else {
        panic!("Expected string, got {:?}", res);
    }

    // time.format_to_epoch
    // args: time_str, format
    let time_str = "2020-09-13 12:26:40";
    let format = "%Y-%m-%d %H:%M:%S";
    let code = format!("time.format_to_epoch('{}', '{}')", time_str, format);
    let res = interp.interpret(&code).unwrap();
    if let Value::Int(ts) = res {
        assert_eq!(ts, 1600000000);
    } else {
        panic!("Expected int, got {:?}", res);
    }

    // time.sleep
    // args: seconds (i64)
    // interpret("time.sleep(0.1)") passes a float.
    // v2 implementation expects i64.
    // If we pass a float, eldritch macros might not auto convert if strictly typed in macro expansion.
    // Or if `eldritch-macros` handle it.
    // However, Rust signature is `i64`.
    // Let's pass an integer. 0.1 cast to int is 0.

    let res = interp.interpret("time.sleep(0)");
    assert!(res.is_ok());
}
