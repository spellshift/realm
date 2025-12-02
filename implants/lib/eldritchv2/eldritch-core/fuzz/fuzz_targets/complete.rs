#![no_main]
use libfuzzer_sys::fuzz_target;
use eldritch_core::{Interpreter, Printer, Span};
use std::sync::Arc;

#[derive(Debug)]
struct NoOpPrinter;

impl Printer for NoOpPrinter {
    fn print_out(&self, _span: &Span, _s: &str) {}
    fn print_err(&self, _span: &Span, _s: &str) {}
}

fuzz_target!(|data: &str| {
    // Generate a random cursor position based on data length
    // Since fuzz_target gives us data, we can't easily get a separate random number for cursor
    // unless we split data.
    // Let's take the first byte as the relative cursor position ratio.

    if data.len() < 1 {
        return;
    }

    let ratio = data.as_bytes()[0] as usize;
    // ratio is 0..255. map to 0..data.len()
    let cursor = (data.len() * ratio) / 255;

    let interpreter = Interpreter::new_with_printer(Arc::new(NoOpPrinter));
    let _ = interpreter.complete(data, cursor);
});
