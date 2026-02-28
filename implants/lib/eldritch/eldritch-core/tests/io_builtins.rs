extern crate alloc;

use alloc::sync::Arc;
use eldritch_core::{BufferPrinter, Interpreter};

fn check_output(code: &str, expected_out: &str, expected_err: &str) {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());

    // Simple indentation stripping
    let code_trimmed = code
        .lines()
        .map(|l| l.trim())
        .collect::<alloc::vec::Vec<_>>()
        .join("\n");

    if let Err(e) = interp.interpret(&code_trimmed) {
        panic!("Interpretation failed for code:\n{}\nError: {}", code, e);
    }

    let out = printer.read_out();
    let err = printer.read_err();

    assert_eq!(out.trim(), expected_out.trim(), "Stdout mismatch");
    assert_eq!(err.trim(), expected_err.trim(), "Stderr mismatch");
}

#[test]
fn test_print_basic() {
    let code = r#"
    print("hello")
    print("world")
    "#;
    check_output(code, "hello\nworld", "");
}

#[test]
fn test_print_multiple_args() {
    let code = r#"
    print("hello", "world", 123)
    "#;
    // print joins args with space
    check_output(code, "hello world 123", "");
}

#[test]
fn test_eprint_basic() {
    let code = r#"
    eprint("error message")
    "#;
    check_output(code, "", "error message");
}

#[test]
fn test_eprint_mixed() {
    let code = r#"
    print("standard")
    eprint("error")
    print("output")
    "#;
    check_output(code, "standard\noutput", "error");
}

#[test]
fn test_pprint_basic() {
    let code = r#"
    d = {"a": 1}
    pprint(d)
    "#;
    // Simple indentation stripping
    let code_trimmed = code
        .lines()
        .map(|l| l.trim())
        .collect::<alloc::vec::Vec<_>>()
        .join("\n");

    // pprint output format depends on implementation, but typically valid JSON/Python dict syntax
    // We expect it to be printed to stdout
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());
    interp.interpret(&code_trimmed).unwrap();

    let out = printer.read_out();
    assert!(out.contains("\"a\": 1") || out.contains("'a': 1"));
}

#[test]
fn test_print_types() {
    let code = r#"
    print(None)
    print(True)
    print([1, 2])
    "#;
    check_output(code, "None\nTrue\n[1, 2]", "");
}
