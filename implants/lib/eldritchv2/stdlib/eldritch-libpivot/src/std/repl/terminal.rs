use crossterm::{
    QueueableCommand, cursor,
    style::{Color, SetForegroundColor},
    terminal,
};
use eldritch_repl::Repl;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest};
use alloc::vec::Vec;

pub struct VtWriter {
    pub tx: std::sync::mpsc::Sender<ReverseShellRequest>,
    pub task_id: i64,
}

impl std::io::Write for VtWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let data = buf.to_vec();
        match self.tx.send(ReverseShellRequest {
            kind: ReverseShellMessageKind::Data.into(),
            data,
            task_id: self.task_id,
        }) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.tx.send(ReverseShellRequest {
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

    if let Some(old) = old_buffer {
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
                stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                stdout.flush()?;
                return Ok(());
            } else if added_len == 0 {
                if state.buffer == *old {
                    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                    stdout.flush()?;
                    return Ok(());
                }
            }
        }

        if state.buffer.len() < old.len()
            && old.starts_with(&state.buffer)
            && state.cursor == state.buffer.len()
            && state.suggestions.is_none()
        {
            let removed_part = &old[state.buffer.len()..];
            if !removed_part.contains('\n') {
                for _ in removed_part.chars() {
                    stdout.write_all(b"\x08 \x08")?;
                }
                stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                stdout.flush()?;
                return Ok(());
            }
        }
    }

    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    stdout.write_all(b"\r")?;

    stdout.queue(SetForegroundColor(Color::Blue))?;
    stdout.write_all(state.prompt.as_bytes())?;
    stdout.queue(SetForegroundColor(Color::Reset))?;

    let buffer_crlf = state.buffer.replace('\n', "\r\n");
    stdout.write_all(buffer_crlf.as_bytes())?;

    if let Some(suggestions) = &state.suggestions {
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
