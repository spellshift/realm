#![no_main]
use libfuzzer_sys::fuzz_target;
use eldritch_core::{Interpreter, Printer};
use std::sync::Arc;

#[derive(Debug)]
struct NoOpPrinter;

impl Printer for NoOpPrinter {
    fn print_out(&self, _s: &str) {}
    fn print_err(&self, _s: &str) {}
}

fuzz_target!(|data: &str| {
    let mut interpreter = Interpreter::new_with_printer(Arc::new(NoOpPrinter));
    // We ignore the result, we just want to ensure it doesn't panic
    let _ = interpreter.interpret(data);
});
