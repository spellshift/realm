use eldritch::{Printer, Span};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum OutputKind {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct StreamPrinter {
    tx: UnboundedSender<(OutputKind, String)>,
}

impl StreamPrinter {
    pub fn new(tx: UnboundedSender<(OutputKind, String)>) -> Self {
        Self { tx }
    }
}

impl Printer for StreamPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior which separates lines
        let _ = self.tx.send((OutputKind::Stdout, format!("{}\n", s)));
    }

    fn print_err(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior
        let _ = self.tx.send((OutputKind::Stderr, format!("{}\n", s)));
    }
}
