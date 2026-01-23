#![allow(dead_code)]
use eldritch_core::{Interpreter, Value};

fn clean_code(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find first non-empty line to determine indent
    let mut indent = None;
    for line in &lines {
        if !line.trim().is_empty() {
            // Calculate indentation by counting leading spaces
            let leading_spaces = line.len() - line.trim_start().len();
            indent = Some(leading_spaces);
            break;
        }
    }

    let indent = indent.unwrap_or(0);
    let mut cleaned = String::new();

    for line in lines {
        if line.len() >= indent {
            // check if the prefix is actually spaces?
            // Assuming mixed tabs/spaces isn't an issue for now, usually it's spaces in Rust raw strings
            cleaned.push_str(&line[indent..]);
        } else {
            cleaned.push_str(line.trim());
        }
        cleaned.push('\n');
    }
    cleaned
}

pub fn code(code: &str) -> Value {
    let cleaned = clean_code(code);
    let mut i = Interpreter::new();
    match i.interpret(&cleaned) {
        Ok(v) => v,
        Err(e) => panic!("Interpreter error in code '{code}': {e}"),
    }
}

pub fn pass(code: &str) {
    let cleaned = clean_code(code);
    let mut i = Interpreter::new();
    if let Err(e) = i.interpret(&cleaned) {
        panic!("Interpreter error in code '{code}': {e}");
    }
}

pub fn all_true(code: &str) {
    let cleaned = clean_code(code);
    let mut i = Interpreter::new();

    // Split by lines and execute each independently
    // This assumes all_true is only used for list of independent assertions
    for line in cleaned.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match i.interpret(line) {
            Ok(v) => match v {
                Value::Bool(b) => {
                    if !b {
                        panic!("Assertion failed for line: {line}");
                    }
                }
                Value::None => {
                    // Ignore None results (assignments, etc)
                }
                _ => panic!("Expected boolean for line '{line}', got {v:?}"),
            },
            Err(e) => panic!("Interpreter error in line '{line}': {e}"),
        }
    }
}

pub fn fail(code: &str, msg_part: &str) {
    let cleaned = clean_code(code);
    let mut i = Interpreter::new();
    match i.interpret(&cleaned) {
        Ok(v) => panic!("Expected error containing '{msg_part}', but got value: {v:?}"),
        Err(e) => {
            if !e.contains(msg_part) {
                panic!("Expected error containing '{msg_part}', but got: '{e}' for code: '{code}'");
            }
        }
    }
}
