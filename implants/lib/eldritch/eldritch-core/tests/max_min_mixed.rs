#[cfg(test)]
mod tests {
    use eldritch_core::{Interpreter, Value};

    #[test]
    fn test_max_min_mixed_types() {
        let code = r#"
tests = [
    [1, 0.8, 1],
    [1, 1.8, 1.8],
    [10, 9.999, 10],
    [10, 19.999, 19.999],
    [1, 1, 1],
    [1, 10, 10],
    [11, 0, 11],
    [.1, 0, .1],
    [.2, .1, .2],
    [.2, 2, 2],
]
errors = []
for a, b, c in tests:
    r = max(a, b)
    if r != c:
        errors.append(f"FAIL max({a}, {b}) != '{c}' got '{r}'")

tests = [
    [1, 0.8, 0.8],
    [1, 1.8, 1],
    [10, 9.999, 9.999],
    [10, 19.999, 10],
    [1, 1, 1],
    [1, 10, 1],
    [11, 0, 0],
    [.1, 0, 0],
    [.2, .1, .1],
    [.2, 2, .2],
]
for a, b, c in tests:
    r = min(a, b)
    if r != c:
        errors.append(f"FAIL min({a}, {b}) != '{c}' got '{r}'")

errors
"#;
        let mut interp = Interpreter::new();
        match interp.interpret(code) {
            Ok(val) => match val {
                Value::List(l) => {
                    let errors = l.read();
                    if !errors.is_empty() {
                        for err in errors.iter() {
                            println!("{}", err);
                        }
                        panic!("Test failed with {} errors", errors.len());
                    }
                }
                _ => panic!("Expected list of errors"),
            },
            Err(e) => panic!("Interpreter error: {}", e),
        }
    }
}
