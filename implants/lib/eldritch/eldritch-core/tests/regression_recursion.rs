use eldritch_core::{Interpreter, Printer, Span};
use std::sync::{Arc, Mutex};

// Mock printer to capture output
#[derive(Debug, Default)]
struct MockPrinter {
    output: Arc<Mutex<String>>,
}

impl Printer for MockPrinter {
    fn print_out(&self, _span: &Span, msg: &str) {
        let mut out = self.output.lock().unwrap();
        out.push_str(msg);
        out.push('\n');
    }
    fn print_err(&self, _span: &Span, msg: &str) {
        let mut out = self.output.lock().unwrap();
        out.push_str("ERR: ");
        out.push_str(msg);
        out.push('\n');
    }
}

#[test]
fn test_recursive_equality_deadlock() {
    let mut interp = Interpreter::new();
    let printer = Arc::new(MockPrinter::default());
    interp.env.write().printer = printer.clone();

    // a = []
    // b = []
    // a.append(b)
    // b.append(a)
    // print(a == b)

    // Note: a == b returns True with our cycle detection (assume equal until proven otherwise).
    // The main goal is to ensure this DOES NOT stack overflow.

    let code = r#"
a = []
b = []
a.append(b)
b.append(a)
print("Comparing...")
x = (a == b)
print(x)
"#;

    let result = interp.interpret(code);
    if let Err(e) = result {
        panic!("Interpreter failed: {:?}", e);
    }

    let output = printer.output.lock().unwrap();
    assert!(output.contains("True"));
}

#[test]
fn test_recursive_list_print() {
    let mut interp = Interpreter::new();
    let printer = Arc::new(MockPrinter::default());
    interp.env.write().printer = printer.clone();

    let code = r#"
a = []
for i in range(1, 10):
    a.append(a)
print(a)
"#;

    let result = interp.interpret(code);
    if let Err(e) = result {
        panic!("Interpreter failed: {:?}", e);
    }

    let output = printer.output.lock().unwrap();
    assert!(output.contains("[...]"));
}
