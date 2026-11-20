use eldritch_core::{Interpreter, Value};

const PRELUDE: &str = include_str!(
    "../../../../../tavern/internal/builder/executor/testdata/input_params_prelude.eldritch"
);

#[test]
fn test_prepended_input_params_line_round_trips() {
    let mut interp = Interpreter::new();
    interp.interpret("input_params = {}").unwrap();
    interp.interpret(PRELUDE).unwrap();

    let expected = [
        ("a", "line1\nline2\ttab\r"),
        ("b", "say \"hi\" \\ back"),
        ("c", "\u{1}\u{7f}"),
        ("d", "nul\0end"),
        ("e", "trail\\"),
        ("k\"ey\\\n", "v"),
        ("u", "héllo 世界 😀 <&>"),
    ];
    match interp.interpret("len(input_params)") {
        Ok(Value::Int(n)) => assert_eq!(n as usize, expected.len()),
        other => panic!("len: unexpected {:?}", other),
    }
    for (k, v) in expected {
        let key = k
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        match interp.interpret(&format!("input_params[\"{}\"]", key)) {
            Ok(Value::String(s)) => assert_eq!(s, v, "key {:?}", k),
            other => panic!("key {:?}: unexpected {:?}", k, other),
        }
    }
}
