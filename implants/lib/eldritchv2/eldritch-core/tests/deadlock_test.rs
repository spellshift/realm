use eldritch_core::Interpreter;
use std::thread;
use std::time::Duration;

#[test]
fn test_list_extend_self_deadlock() {
    // We use a thread to detect the hang. If it finishes quickly, it passed.
    // If it hangs, the main thread will panic/timeout.
    // We'll wrap the execution in a channel.

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let mut interp = Interpreter::new();
        let code = r#"
l = [1, 2, 3]
l.extend(l)
"#;
        let res = interp.interpret(code);
        tx.send(res).unwrap();
    });

    // Wait for result with a timeout
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(result) => {
            assert!(result.is_ok(), "Interpretation failed: {:?}", result.err());
            // Verify content if needed, but the fact that it finished is the main check.
        }
        Err(_) => {
            panic!("Test timed out! Likely deadlock in l.extend(l)");
        }
    }
}
