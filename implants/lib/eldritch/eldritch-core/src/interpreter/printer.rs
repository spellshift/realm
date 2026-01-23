use crate::token::Span;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;
use spin::Mutex;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::{eprintln, println};

/// Trait for handling output from the interpreter.
/// It must be Send + Sync to be safe in threaded environments.
/// It must implement Debug to satisfy AST derivation requirements.
pub trait Printer: Send + Sync + fmt::Debug {
    /// Print to standard output
    fn print_out(&self, span: &Span, s: &str);

    /// Print to standard error
    fn print_err(&self, span: &Span, s: &str);
}

/// A printer that writes to standard output (println!).
/// In no_std environments without the "std" feature, this is a no-op.
#[derive(Debug)]
pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        #[cfg(feature = "std")]
        {
            println!("{s}");
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = s;
        }
    }

    fn print_err(&self, _span: &Span, s: &str) {
        #[cfg(feature = "std")]
        {
            eprintln!("{s}");
        }
        #[cfg(not(feature = "std"))]
        {
            let _ = s;
        }
    }
}

/// A printer that writes to an internal string buffer.
/// Useful for capturing output in tests or REPL environments.
#[derive(Debug)]
pub struct BufferPrinter {
    stdout: Arc<Mutex<String>>,
    stderr: Arc<Mutex<String>>,
}

impl BufferPrinter {
    pub fn new() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(String::new())),
            stderr: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn get_stdout(&self) -> Arc<Mutex<String>> {
        self.stdout.clone()
    }

    pub fn get_stderr(&self) -> Arc<Mutex<String>> {
        self.stderr.clone()
    }

    pub fn clear(&self) {
        self.stdout.lock().clear();
        self.stderr.lock().clear();
    }

    pub fn read(&self) -> String {
        let out = self.stdout.lock();
        let err = self.stderr.lock();
        if err.is_empty() {
            out.clone()
        } else if out.is_empty() {
            err.clone()
        } else {
            // Concatenate if both exist
            alloc::format!("{}\n{}", *out, *err)
        }
    }

    pub fn read_out(&self) -> String {
        self.stdout.lock().clone()
    }

    pub fn read_err(&self) -> String {
        self.stderr.lock().clone()
    }
}

impl Default for BufferPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Printer for BufferPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        let mut buf = self.stdout.lock();
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(s);
    }

    fn print_err(&self, _span: &Span, s: &str) {
        let mut buf = self.stderr.lock();
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(s);
    }
}
