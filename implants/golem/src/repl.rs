use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Stylize,
    terminal::{self, ClearType},
};
use eldritch::repl::{Input, Repl, ReplAction};
use eldritch::{Interpreter, Value};
use std::io::{self, Write};
use std::time::Duration;

pub fn repl(mut inter: Interpreter) -> io::Result<()> {
    let mut repl = Repl::new();

    // Register STD-dependent builtins
    inter.register_module(
        "input",
        Value::NativeFunction("input".to_string(), |_env, _| {
            terminal::disable_raw_mode().unwrap();
            let mut input = String::new();
            let res = match std::io::stdin().read_line(&mut input) {
                Ok(_) => Ok(Value::String(input.trim().to_string())),
                Err(e) => Err(format!("Input error: {e}")),
            };
            terminal::enable_raw_mode().unwrap();
            res
        }),
    );

    // Add a way out
    inter.register_module(
        "exit",
        Value::NativeFunction("exit".to_string(), |_env, _| {
            println!();
            std::process::exit(0)
        }),
    );

    println!("Type 'exit()' to quit. End blocks with an empty line.");
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    render(&mut stdout, &repl)?;

    loop {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            let input = map_key(key);
            if let Some(input) = input {
                match repl.handle_input(input) {
                    ReplAction::Quit => break,
                    ReplAction::Submit {
                        code,
                        last_line: _,
                        prompt: _,
                    } => {
                        // Clear current line visual and move down
                        stdout.execute(cursor::MoveToNextLine(1))?;

                        terminal::disable_raw_mode()?;
                        match inter.interpret(&code) {
                            Ok(v) => {
                                if !matches!(v, Value::None) {
                                    println!("{v:?}");
                                }
                            }
                            Err(e) => println!("Error: {e}"),
                        }
                        terminal::enable_raw_mode()?;

                        render(&mut stdout, &repl)?;
                    }
                    ReplAction::AcceptLine { line: _, prompt: _ } => {
                        stdout.execute(cursor::MoveToNextLine(1))?;
                        render(&mut stdout, &repl)?;
                    }
                    ReplAction::Render => {
                        render(&mut stdout, &repl)?;
                    }
                    ReplAction::ClearScreen => {
                        stdout.execute(terminal::Clear(ClearType::All))?;
                        stdout.execute(cursor::MoveTo(0, 0))?;
                        render(&mut stdout, &repl)?;
                    }
                    ReplAction::Complete => {
                        let state = repl.get_render_state();
                        let (start, completions) = inter.complete(&state.buffer, state.cursor);
                        repl.set_suggestions(completions, start);
                        render(&mut stdout, &repl)?;
                    }
                    ReplAction::None => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

pub(crate) fn map_key(key: KeyEvent) -> Option<Input> {
    // Windows consoles emit both key-down and key-up input records for every
    // keypress, and crossterm's Windows event source surfaces both as
    // `Event::Key` (`KeyEventKind::Press` and `KeyEventKind::Release`), with
    // no filtering. Unix only ever emits `Press`. Without this check, every
    // typed character is processed twice on Windows (e.g. `print("hello")`
    // becomes `pprriinntt((""hheelllloo""))`).
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::Cancel),
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::ClearScreen)
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::EOF),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::KillLine)
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::KillToEnd)
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::WordBackspace)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::HistorySearch)
        }
        KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Input::ForceComplete)
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => Some(Input::ForceEnter),
        KeyCode::Char(c) => Some(Input::Char(c)),
        KeyCode::Enter => Some(Input::Enter),
        KeyCode::Backspace => Some(Input::Backspace),
        KeyCode::Delete => Some(Input::Delete),
        KeyCode::Left => Some(Input::Left),
        KeyCode::Right => Some(Input::Right),
        KeyCode::Up => Some(Input::Up),
        KeyCode::Down => Some(Input::Down),
        KeyCode::Home => Some(Input::Home),
        KeyCode::End => Some(Input::End),
        KeyCode::Tab => Some(Input::Tab),
        _ => None,
    }
}

fn render(stdout: &mut io::Stdout, repl: &Repl) -> io::Result<()> {
    let state = repl.get_render_state();

    // Clear everything below the current line to clear old suggestions
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    stdout.queue(cursor::MoveToColumn(0))?;

    let full_line = format!("{}{}", state.prompt.as_str().blue(), state.buffer);
    stdout.write_all(full_line.as_bytes())?;

    // Render suggestions if any
    if let Some(suggestions) = &state.suggestions {
        // Save cursor position
        stdout.queue(cursor::SavePosition)?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(0))?;

        // Print suggestions
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
                // i is the absolute index
                // Separator logic: if we are not the very first item displayed
                if i > start {
                    stdout.write_all(b"  ")?;
                }

                if Some(i) == state.suggestion_idx {
                    // Highlight selected
                    stdout.write_all(format!("{}", s.as_str().black().on_white()).as_bytes())?;
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
    use crossterm::event::KeyEventState;

    fn key_with_kind(c: char, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind_and_state(
            KeyCode::Char(c),
            KeyModifiers::NONE,
            kind,
            KeyEventState::NONE,
        )
    }

    // Regression test for https://github.com/spellshift/realm/issues/1702:
    // on Windows, crossterm surfaces both key-down (Press) and key-up
    // (Release) console input records as `Event::Key`, which previously
    // caused every typed character to be processed twice. `map_key` must
    // only act on `Press` events.
    #[test]
    fn map_key_only_handles_press_events() {
        let press = key_with_kind('a', KeyEventKind::Press);
        assert_eq!(map_key(press), Some(Input::Char('a')));

        let release = key_with_kind('a', KeyEventKind::Release);
        assert_eq!(map_key(release), None);

        let repeat = key_with_kind('a', KeyEventKind::Repeat);
        assert_eq!(map_key(repeat), None);
    }

    // Held keys must still auto-repeat: without keyboard-enhancement mode
    // (which golem does not enable), a held key arrives as repeated `Press`
    // events followed by a single `Release`. Every press must yield input.
    #[test]
    fn map_key_held_key_auto_repeat() {
        let inputs: Vec<Input> = (0..5)
            .map(|_| key_with_kind('x', KeyEventKind::Press))
            .chain(std::iter::once(key_with_kind('x', KeyEventKind::Release)))
            .filter_map(map_key)
            .collect();
        assert_eq!(inputs.len(), 5);
        assert!(inputs.iter().all(|i| *i == Input::Char('x')));
    }

    // Simulates a real Windows console event stream for typing "hello":
    // (press, release) per keystroke, including the doubled "ll". Each press
    // must yield exactly one input and each release none, so fast repeated
    // characters keep working without duplicating.
    #[test]
    fn map_key_windows_press_release_stream_yields_single_inputs() {
        use KeyEventKind::{Press, Release};
        let inputs: Vec<Input> = ['h', 'e', 'l', 'l', 'o']
            .into_iter()
            .flat_map(|c| [key_with_kind(c, Press), key_with_kind(c, Release)])
            .filter_map(map_key)
            .collect();
        let expected: Vec<Input> = ['h', 'e', 'l', 'l', 'o']
            .into_iter()
            .map(Input::Char)
            .collect();
        assert_eq!(inputs, expected);
    }

    #[test]
    fn map_key_release_does_not_double_process_control_keys() {
        let enter_press = KeyEvent::new_with_kind_and_state(
            KeyCode::Enter,
            KeyModifiers::NONE,
            KeyEventKind::Press,
            KeyEventState::NONE,
        );
        assert_eq!(map_key(enter_press), Some(Input::Enter));

        let enter_release = KeyEvent::new_with_kind_and_state(
            KeyCode::Enter,
            KeyModifiers::NONE,
            KeyEventKind::Release,
            KeyEventState::NONE,
        );
        assert_eq!(map_key(enter_release), None);
    }
}
