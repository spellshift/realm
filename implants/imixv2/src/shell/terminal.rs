use crossterm::{
    cursor,
    style::{Color, SetForegroundColor},
    terminal, QueueableCommand,
};
use eldritch_repl::Repl;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest};

pub struct VtWriter {
    pub tx: tokio::sync::mpsc::Sender<ReverseShellRequest>,
    pub task_id: i64,
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

pub fn render<W: std::io::Write>(stdout: &mut W, repl: &Repl) -> std::io::Result<()> {
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
    use eldritch_repl::Input;
    use std::io::Write; // Needed here for the test helper vector write

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
