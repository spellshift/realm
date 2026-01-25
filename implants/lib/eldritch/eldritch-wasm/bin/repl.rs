use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal::{self, ClearType},
};
use eldritch::Interpreter;
use eldritch_core::Value;
use eldritch_repl::{Input, Repl, ReplAction};
use std::io::{self, Write};
use std::time::Duration;

#[cfg(feature = "fake_bindings")]
use eldritch::{
    crypto::fake::CryptoLibraryFake, file::fake::FileLibraryFake, http::fake::HttpLibraryFake,
    regex::fake::RegexLibraryFake,
};

fn main() -> io::Result<()> {
    // Register Libraries logic is now handled by Interpreter builder

    let mut interpreter = Interpreter::new();

    #[cfg(feature = "stdlib")]
    {
        interpreter = interpreter.with_default_libs();
    }

    #[cfg(all(not(feature = "stdlib"), feature = "fake_bindings"))]
    {
        use std::sync::Arc;
        interpreter.inner.register_lib(FileLibraryFake::default());
        interpreter.inner.register_lib(HttpLibraryFake::default());
        interpreter.inner.register_lib(RegexLibraryFake::default());
        interpreter.inner.register_lib(CryptoLibraryFake::default());
    }

    let mut repl = Repl::new();

    // Register STD-dependent builtins
    interpreter.register_module(
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
    // Note: register_function became register_module? No, eldritch wrapped register_module but not register_function directly?
    // Let me check my implementation of eldritch::Interpreter.
    // I did NOT expose register_function. I exposed register_module.
    // But wait, `interpreter.inner.register_function` exists.
    // In `eldritch/src/lib.rs`, I didn't verify if I exposed `register_function`.
    // I checked: I implemented `register_module`. I did NOT implement `register_function`.
    // The previous code used `register_function` for "input".
    // I should fix `eldritch/src/lib.rs` to expose `register_function` or use `register_module` with a dict or just native function value.
    // `Interpreter::register_function` in core does: `self.env.write().values.insert(name, Value::NativeFunction(...))`.
    // `Interpreter::register_module` does: `self.env.write().values.insert(name, module)`.
    // So `register_module` can take a `Value::NativeFunction` and it works the same (inserts into env).
    // So `interpreter.register_module("input", Value::NativeFunction(...))` works.

    println!("Eldritch REPL (Rust + Crossterm)");
    println!("Type 'exit' to quit. End blocks with an empty line.");

    // Load history
    if let Ok(content) = std::fs::read_to_string("eldritch_history.txt") {
        let history: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        repl.load_history(history);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    render(&mut stdout, &repl)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
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
                            match interpreter.interpret(&code) {
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
                            let (start, completions) =
                                interpreter.complete(&state.buffer, state.cursor);
                            repl.set_suggestions(completions, start);
                            render(&mut stdout, &repl)?;
                        }
                        ReplAction::None => {}
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;

    let history = repl.get_history();
    let content = history.join("\n");
    std::fs::write("eldritch_history.txt", content)?;

    Ok(())
}

fn map_key(key: KeyEvent) -> Option<Input> {
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
