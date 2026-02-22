use eldritch::{Printer, Span};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct StreamPrinter {
    tx: UnboundedSender<String>,
    error_tx: UnboundedSender<String>,
}

impl StreamPrinter {
    pub fn new(tx: UnboundedSender<String>, error_tx: UnboundedSender<String>) -> Self {
        Self { tx, error_tx }
    }
}

impl Printer for StreamPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior which separates lines
        let _ = self.tx.send(format!("{}\n", s));
    }

    fn print_err(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior
        let _ = self.error_tx.send(format!("{}\n", s));
    }
}
