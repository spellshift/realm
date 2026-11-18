use eldritch_core::{Interpreter, Value};

#[test]
fn test_prepended_input_params_line_round_trips() {
    let line = "input_params = {\"a\": \"line1\\nline2\\ttab\\r\", \"b\": \"say \\\"hi\\\" \\\\ back\", \"c\": \"\\x01\\x7f\", \"u\": \"héllo 世界 😀 <&>\"}\n";
    let mut interp = Interpreter::new();
    interp.interpret("input_params = {}").unwrap();
    interp.interpret(line).unwrap();

    let expected = [
        ("a", "line1\nline2\ttab\r"),
        ("b", "say \"hi\" \\ back"),
        ("c", "\u{1}\u{7f}"),
        ("u", "héllo 世界 😀 <&>"),
    ];
    for (k, v) in expected {
        match interp.interpret(&format!("input_params[\"{}\"]", k)) {
            Ok(Value::String(s)) => assert_eq!(s, v, "key {}", k),
            other => panic!("key {}: unexpected {:?}", k, other),
        }
    }
}
