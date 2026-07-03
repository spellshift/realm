extern crate alloc;

use alloc::sync::Arc;
use eldritch_core::{BufferPrinter, Interpreter};

fn check_output_contains(code: &str, expected: &[&str]) {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());

    let code_trimmed = code
        .lines()
        .map(|l| l.trim())
        .collect::<alloc::vec::Vec<_>>()
        .join("\n");

    if let Err(e) = interp.interpret(&code_trimmed) {
        panic!("Interpretation failed for code:\n{}\nError: {}", code, e);
    }

    let output = printer.read();

    for fragment in expected {
        if !output.contains(fragment) {
            panic!(
                "Output did not contain '{}'. Output was:\n{}",
                fragment, output
            );
        }
    }
}

fn check_error(code: &str, error_fragment: &str) {
    let mut interp = Interpreter::new();
    match interp.interpret(code) {
        Ok(_) => panic!(
            "Expected error containing '{}', but succeeded.",
            error_fragment
        ),
        Err(e) => {
            if !e.contains(error_fragment) {
                panic!(
                    "Expected error containing '{}', but got: '{}'",
                    error_fragment, e
                );
            }
        }
    }
}

#[test]
fn test_tprint_invalid_args() {
    check_error("tprint([1, 2])", "must contain only dictionaries");
}

#[test]
fn test_tprint_success() {
    check_output_contains("tprint([{\"a\": 1}])", &["| a |", "| 1 |"]);
}

#[test]
fn test_tprint_empty_args() {
    check_error("tprint()", "takes at least 1 argument");
}

#[test]
fn test_tprint_invalid_type() {
    check_error("tprint(1)", "argument must be a list");
}
