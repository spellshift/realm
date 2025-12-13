use eldritch_core::Interpreter;

#[test]
fn test_error_caret_alignment() {
    let mut interp = Interpreter::new();

    // Test case 1: Error not at start, with indentation
    // "    x = z + 1"
    //      ^   ^
    //      0   4 (relative to trimmed)
    // In raw string: 4 spaces + 'x' + ' ' + '=' + ' ' + 'z'
    // 'z' is at index 8.
    // Trimmed line: "x = z + 1" (starts at index 4)
    // Error at 8. Relative to trimmed start: 8 - 4 = 4.
    // Display indent: 4 (base) + 4 (relative) = 8 spaces.

    let code = "    x = z + 1";
    let res = interp.interpret(code);
    match res {
        Ok(_) => panic!("Expected error for 'x = z + 1'"),
        Err(msg) => {
            let lines: Vec<&str> = msg.lines().collect();
            // Expected output:
            // ...
            // Error location:
            //   at line 1:
            //     x = z + 1
            //         ^-- here

            let last_line = lines.last().expect("Error message empty");
            // "        ^-- here"
            // 8 spaces + ^
            assert!(
                last_line.starts_with("        ^-- here"),
                "Incorrect alignment: '{}'",
                last_line
            );
        }
    }

    // Test case 2: Error at start of trimmed line
    // Just accessing undefined 'z'
    let code2 = "    z";
    let res2 = interp.interpret(code2);
    match res2 {
        Ok(_) => panic!("Expected error for 'z'"),
        Err(msg) => {
            let lines: Vec<&str> = msg.lines().collect();
            let last_line = lines.last().expect("Error message empty");
            // "    ^-- here" (4 spaces indent)
            assert!(
                last_line.starts_with("    ^-- here"),
                "Incorrect alignment for start of line: '{}'",
                last_line
            );
        }
    }
}
