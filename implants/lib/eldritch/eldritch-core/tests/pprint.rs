extern crate alloc;
mod assert;

use alloc::sync::Arc;
use eldritch_core::{BufferPrinter, Interpreter};

fn clean_code(code: &str) -> alloc::string::String {
    let lines: alloc::vec::Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return alloc::string::String::new();
    }

    let mut indent = None;
    for line in &lines {
        if !line.trim().is_empty() {
            let leading_spaces = line.len() - line.trim_start().len();
            indent = Some(leading_spaces);
            break;
        }
    }

    let indent = indent.unwrap_or(0);
    let mut cleaned = alloc::string::String::new();

    for line in lines {
        if line.len() >= indent {
            cleaned.push_str(&line[indent..]);
        } else {
            cleaned.push_str(line.trim());
        }
        cleaned.push('\n');
    }
    cleaned
}

fn check_output(code: &str, expected_out: &str, expected_err: &str) {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());

    let code_cleaned = clean_code(code);

    if let Err(e) = interp.interpret(&code_cleaned) {
        panic!("Interpretation failed for code:\n{}\nError: {}", code, e);
    }

    let out = printer.read_out();
    let err = printer.read_err();

    assert_eq!(out.trim(), expected_out.trim(), "Stdout mismatch");
    assert_eq!(err.trim(), expected_err.trim(), "Stderr mismatch");
}

fn check_error(code: &str, expected_err: &str) {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());

    let code_cleaned = clean_code(code);

    match interp.interpret(&code_cleaned) {
        Ok(_) => panic!("Expected error but succeeded"),
        Err(e) => {
            let e_str = alloc::format!("{}", e);
            assert!(
                e_str.contains(expected_err),
                "Error '{}' didn't contain '{}'",
                e_str,
                expected_err
            );
        }
    }
}

#[test]
fn test_pprint_collections() {
    check_output("pprint([1, 2, 3])", "[\n  1,\n  2,\n  3\n]", "");
    check_output("pprint([])", "[]", "");

    check_output(
        r#"pprint({"a": 1, "b": 2})"#,
        "{\n  \"a\": 1,\n  \"b\": 2\n}",
        "",
    );
    check_output("pprint({})", "{}", "");

    check_output("pprint((1, 2, 3))", "(\n  1,\n  2,\n  3\n)", "");
    check_output("pprint((1,))", "(\n  1,\n)", "");
    check_output("pprint(())", "()", "");

    check_output("pprint(set([1, 2, 3]))", "{\n  1,\n  2,\n  3\n}", "");
    check_output("pprint(set())", "set()", "");
}

#[test]
fn test_pprint_nested() {
    check_output(
        r#"pprint([1, {"a": [2]}, 3])"#,
        "[\n  1,\n  {\n    \"a\": [\n      2\n    ]\n  },\n  3\n]",
        "",
    );
}

#[test]
fn test_pprint_types() {
    check_output("pprint('test')", "\"test\"", "");
    check_output("pprint(123)", "123", "");
    check_output("pprint(True)", "True", "");
    check_output("pprint(None)", "None", "");
}

#[test]
fn test_pprint_indent() {
    check_output("pprint([1, 2], 4)", "[\n    1,\n    2\n]", "");
    check_output("pprint([1, 2], -1)", "[\n1,\n2\n]", "");
}

#[test]
fn test_pprint_errors() {
    check_error("pprint()", "takes at least 1 argument");
    check_error("pprint([1], 'a')", "indent must be an integer");
}
