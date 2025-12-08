use eldritch_core::{Interpreter, Value};
use eldritch_librandom::std::StdRandomLibrary;

#[test]
fn test_random() {
    let lib = StdRandomLibrary::default();
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // random.int(min, max)
    // The previous error said "Missing argument: max", so it expects 2 args.
    let res = interp.interpret("random.int(0, 100)").unwrap();
    if let Value::Int(i) = res {
        assert!(i >= 0 && i < 100);
    } else {
        panic!("Expected int, got {:?}", res);
    }

    // random.string
    let res = interp.interpret("random.string(10)").unwrap();
    if let Value::String(s) = res {
        assert_eq!(s.len(), 10);
    } else {
        panic!("Expected string, got {:?}", res);
    }

    // random.bool
    let res = interp.interpret("random.bool()").unwrap();
    if let Value::Bool(_) = res {
        // ok
    } else {
        panic!("Expected bool, got {:?}", res);
    }

    // random.uuid
    let res = interp.interpret("random.uuid()").unwrap();
    if let Value::String(s) = res {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string for uuid, got {:?}", res);
    }
}
