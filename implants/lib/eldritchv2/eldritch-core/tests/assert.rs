use eldritch_core::Interpreter;

// Helper to remove common leading whitespace from raw strings in tests
fn dedent(code: &str) -> String {
    let lines: Vec<&str> = code
        .lines()
        .skip_while(|l| l.trim().is_empty()) // Skip initial empty lines (often caused by r#" \n ...)
        .collect();

    if lines.is_empty() {
        return String::new();
    }

    // Determine min indentation of non-empty lines
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.len() >= indent {
                &line[indent..]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Asserts that the provided code executes successfully.
pub fn pass(code: &str) {
    let clean_code = dedent(code);
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.interpret(&clean_code) {
        panic!(
            "Expected execution to pass, but it failed.\nError: {}\nCode:\n{}",
            e, clean_code
        );
    }
}

/// Asserts that the provided code fails with an error message containing `msg`.
pub fn fail(code: &str, msg: &str) {
    let clean_code = dedent(code);
    let mut interpreter = Interpreter::new();
    match interpreter.interpret(&clean_code) {
        Ok(val) => panic!(
            "Expected execution to fail with \"{}\", but it succeeded.\nReturned: {:?}\nCode:\n{}",
            msg, val, clean_code
        ),
        Err(e) => {
            if !e.contains(msg) {
                panic!(
                    "Expected error containing \"{}\", but got \"{}\".\nCode:\n{}",
                    msg, e, clean_code
                );
            }
        }
    }
}

/// Asserts that `lhs` evaluates to the same value as `rhs`.
/// Generates code: `assert_eq(lhs, rhs)`
pub fn eq(lhs: &str, rhs: &str) {
    let code = format!("assert_eq({}, {})", lhs, rhs);
    pass(&code);
}

/// Takes a string of boolean expressions (one per line) and asserts they are all True.
/// Wraps each non-empty line in `assert(...)`.
pub fn all_true(code: &str) {
    let clean_code = dedent(code);
    let mut test_script = String::new();
    for line in clean_code.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            test_script.push_str(&format!("assert({})\n", trimmed));
        }
    }
    pass(&test_script);
}
