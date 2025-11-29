extern crate alloc;
mod assert;

use eldritchv2::{interpreter::Interpreter, eldritch_module};
use alloc::string::{String, ToString};

// Updated signatures to match new macro expectations
fn my_add(a: i64, b: i64) -> Result<i64, String> {
    Ok(a + b)
}

fn my_greet(s: String) -> Result<String, String> {
    Ok(alloc::format!("Hello, {}!", s))
}

#[test]
fn test_register_module_macro() {
    let mut interpreter = Interpreter::new();

    let my_lib = eldritch_module! {
        name: "mylib",
        functions: {
            "add" => my_add,
            "greet" => my_greet,
        }
    };

    interpreter.register_module("mylib", my_lib);

    let code = r#"
assert(mylib.add(1, 2) == 3)
assert(mylib.greet("World") == "Hello, World!")
"#;

    if let Err(e) = interpreter.interpret(code) {
        panic!("Execution failed: {}", e);
    }
}

#[test]
fn test_stateful_module() {
    // See macro_conversion_test.rs for more comprehensive tests.
}
