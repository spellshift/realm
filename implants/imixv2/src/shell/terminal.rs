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

pub fn render<W: std::io::Write>(
    stdout: &mut W,
    repl: &Repl,
    old_buffer: Option<&str>,
) -> std::io::Result<()> {
    let state = repl.get_render_state();

    // Optimization: If we are appending characters to the end of the buffer (one or more),
    // and no other state changes (like suggestions) are present, just print the new characters.
    // This avoids clearing and redrawing the whole line, which prevents duplication issues
    // when the line wraps in the terminal (since we don't track terminal width) and avoids
    // flickering when typing fast or pasting long strings.
    if let Some(old) = old_buffer {
        // Append optimization
        if state.buffer.len() >= old.len()
            && state.buffer.starts_with(old)
            && state.cursor == state.buffer.len()
            && state.suggestions.is_none()
        {
            let added_len = state.buffer.len() - old.len();
            if added_len > 0 {
                let new_part = &state.buffer[old.len()..];
                let s_crlf = new_part.replace('\n', "\r\n");
                stdout.write_all(s_crlf.as_bytes())?;
                // Clear any potential suggestions below (e.g. if we just typed a char that closed suggestions)
                stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                stdout.flush()?;
                return Ok(());
            } else if added_len == 0 {
                // Buffer hasn't changed size and we are at the end, maybe cursor move?
                // If content is same, do nothing but clear suggestions
                if state.buffer == *old {
                    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                    stdout.flush()?;
                    return Ok(());
                }
            }
        }

        // Backspace at the end optimization
        // If we removed characters from the end of the buffer, and we are at the end,
        // we can just emit backspaces instead of clearing the line.
        // This is critical for handling long lines that have wrapped, because we don't know
        // the terminal width and clearing the current line only clears the last wrapped segment.
        if state.buffer.len() < old.len()
            && old.starts_with(&state.buffer)
            && state.cursor == state.buffer.len()
            && state.suggestions.is_none()
        {
            let removed_part = &old[state.buffer.len()..];
            // Only optimize if we haven't removed any newlines (which would involve moving cursor up)
            if !removed_part.contains('\n') {
                for _ in removed_part.chars() {
                    // Backspace, Space, Backspace to erase character visually
                    stdout.write_all(b"\x08 \x08")?;
                }
                // Clear any potential suggestions below
                stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                stdout.flush()?;
                return Ok(());
            }
        }
    }

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
    stdout.write_all(b"\r")?;
    if cursor_col > 0 {
        stdout.queue(cursor::MoveRight(cursor_col))?;
    }

    stdout.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eldritch_repl::{Repl, Input};

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
        render(&mut stdout, &repl, None).unwrap();

        let output = String::from_utf8_lossy(&stdout);

        // Check that newlines are converted to \r\n
        // The output will contain clearing codes, prompt colors, etc.
        // We look for the sequence:
        // "for i in range(5):\r\n    print(i)\r\n    print(i*2)"

        assert!(output.contains("for i in range(5):\r\n    print(i)\r\n    print(i*2)"));
    }

    #[test]
    fn test_render_append_multi_char() {
        let mut repl = Repl::new();
        // Setup initial state: "abc"
        repl.handle_input(Input::Char('a'));
        repl.handle_input(Input::Char('b'));
        repl.handle_input(Input::Char('c'));

        // This is what old_buffer was
        let old_buffer = "abc".to_string();

        // Now append "def" (multiple chars)
        repl.handle_input(Input::Char('d'));
        repl.handle_input(Input::Char('e'));
        repl.handle_input(Input::Char('f'));

        // Current buffer is "abcdef"

        let mut stdout = Vec::new();
        render(&mut stdout, &repl, Some(&old_buffer)).unwrap();

        let output = String::from_utf8_lossy(&stdout);

        println!("Output: {:?}", output);

        // If full redraw, we expect to see prompt color seq
        let has_full_redraw = output.contains("\x1b[34m"); // Blue color for prompt

        // We expect NO full redraw, only "def" + clear down
        assert!(!has_full_redraw, "Should NOT fall back to full redraw for multi-char append. Output was: {:?}", output);

        // Output should start with "def"
        assert!(output.starts_with("def"));
    }

    #[test]
    fn test_render_backspace_at_end() {
        let mut repl = Repl::new();
        // Setup initial state: "abc"
        repl.handle_input(Input::Char('a'));
        repl.handle_input(Input::Char('b'));
        repl.handle_input(Input::Char('c'));

        // This is what old_buffer was
        let old_buffer = "abc".to_string();

        // Now backspace
        repl.handle_input(Input::Backspace);

        // Current buffer is "ab"

        let mut stdout = Vec::new();
        render(&mut stdout, &repl, Some(&old_buffer)).unwrap();

        let output = String::from_utf8_lossy(&stdout);

        println!("Output: {:?}", output);

        // If full redraw, we expect to see prompt color seq
        let has_full_redraw = output.contains("\x1b[34m"); // Blue color for prompt

        // We expect NO full redraw, only backspaces
        assert!(!has_full_redraw, "Should NOT fall back to full redraw for backspace at end. Output was: {:?}", output);

        // Output should contain backspace sequence "\x08 \x08" (BS Space BS)
        // Note: crossterm or manual writing might differ, but we expect manual backspace handling
        assert!(output.contains("\x08 \x08"), "Should contain backspace sequence. Output: {:?}", output);
    }
}
