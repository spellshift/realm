#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Unstructured;
use eldritch_core::{Interpreter, Printer, Span};
use std::sync::Arc;

#[derive(Debug)]
struct NoOpPrinter;

impl Printer for NoOpPrinter {
    fn print_out(&self, _span: &Span, _s: &str) {}
    fn print_err(&self, _span: &Span, _s: &str) {}
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let code: String = match u.arbitrary() {
        Ok(s) => s,
        Err(_) => return,
    };
    if code.is_empty() {
        return;
    }

    // Generate a valid cursor position within the string bounds
    let cursor = match u.int_in_range(0..=code.len()) {
        Ok(i) => i,
        Err(_) => return,
    };

    let interpreter = Interpreter::new_with_printer(Arc::new(NoOpPrinter));
    let _ = interpreter.complete(&code, cursor);
});
