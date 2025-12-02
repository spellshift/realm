use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;
use spin::Mutex;

/// Trait for handling output from the interpreter.
/// It must be Send + Sync to be safe in threaded environments.
/// It must implement Debug to satisfy AST derivation requirements.
pub trait Printer: Send + Sync + fmt::Debug {
    fn print(&self, s: &str);
}

/// A printer that writes to standard output (println!).
/// In no_std environments without the "std" feature, this is a no-op.
#[derive(Debug)]
pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
    fn print(&self, s: &str) {
        #[cfg(feature = "std")]
        {
            println!("{}", s);
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
    buffer: Arc<Mutex<String>>,
}

impl BufferPrinter {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(String::new())),
        }
    }

    pub fn get_buffer(&self) -> Arc<Mutex<String>> {
        self.buffer.clone()
    }

    pub fn clear(&self) {
        self.buffer.lock().clear();
    }

    pub fn read(&self) -> String {
        self.buffer.lock().clone()
    }
}

impl Default for BufferPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Printer for BufferPrinter {
    fn print(&self, s: &str) {
        let mut buf = self.buffer.lock();
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(s);
    }
}
