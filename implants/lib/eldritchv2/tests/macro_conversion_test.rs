use eldritchv2::eldritch_module;
use eldritchv2::Interpreter;
use eldritchv2::Value;

extern crate alloc;

// Test function with conversion
fn my_add(x: i64, y: i64) -> Result<i64, alloc::string::String> {
    Ok(x + y)
}

fn my_concat(a: alloc::string::String, b: alloc::string::String) -> Result<alloc::string::String, alloc::string::String> {
    Ok(a + &b)
}

#[test]
fn test_macro_conversion() {
    let module = eldritch_module! {
        name: "mylib",
        functions: {
            "add" => my_add,
            "concat" => my_concat,
        }
    };

    let mut interp = Interpreter::new();
    interp.register_module("mylib", module);

    // Test add
    let res = interp.interpret("mylib.add(10, 20)").unwrap();
    match res {
        Value::Int(i) => assert_eq!(i, 30),
        _ => panic!("Expected Int(30)"),
    }

    // Test concat
    let res = interp.interpret("mylib.concat('Hello, ', 'World!')").unwrap();
    match res {
        Value::String(s) => assert_eq!(s, "Hello, World!"),
        _ => panic!("Expected String('Hello, World!')"),
    }

    // Test error
    let res = interp.interpret("mylib.add(10, 'wrong')");
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("Expected Int"));
}
