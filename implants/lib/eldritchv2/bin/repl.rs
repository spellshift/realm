use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use eldritchv2::{Interpreter, Repl, Value, register_lib};
use eldritchv2::repl::{Input, ReplAction};
use std::io::{self, Write};
use std::time::Duration;

#[cfg(feature = "std")]
use eldritchv2::bindings::process::std::StdProcessLibrary;

fn main() -> io::Result<()> {
    #[cfg(feature = "std")]
    {
        register_lib(StdProcessLibrary::default());
    }

    let mut interpreter = Interpreter::new();
    let mut repl = Repl::new();

    // Register STD-dependent builtins
    interpreter.register_function("print", |_env, args| {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg.to_string());
        }
        println!();
        Ok(Value::None)
    });

    interpreter.register_function("input", |_env, _| {
        terminal::disable_raw_mode().unwrap();
        let mut input = String::new();
        let res = match std::io::stdin().read_line(&mut input) {
            Ok(_) => Ok(Value::String(input.trim().to_string())),
            Err(e) => Err(format!("Input error: {}", e)),
        };
        terminal::enable_raw_mode().unwrap();
        res
    });

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
            match event::read()? {
                Event::Key(key) => {
                    let input = map_key(key);
                    if let Some(input) = input {
                        match repl.handle_input(input) {
                            ReplAction::Quit => break,
                            ReplAction::Submit { code, last_line: _, prompt: _ } => {
                                // Clear current line visual and move down
                                // We don't need to manually print the last line because it's already on screen?
                                // Wait, `repl.handle_input` cleared the buffer.
                                // If we just move down, the *old* buffer is still on the screen.
                                // BUT `render()` will be called next? No.
                                // We are submitting.
                                // The user typed "print(1)" [Enter].
                                // Screen shows ">>> print(1)". Cursor at end.
                                // We receive Submit.
                                // We just need to MoveToNextLine.
                                stdout.execute(cursor::MoveToNextLine(1))?;

                                terminal::disable_raw_mode()?;
                                match interpreter.interpret(&code) {
                                    Ok(v) => {
                                        if !matches!(v, Value::None) {
                                            println!("{:?}", v);
                                        }
                                    }
                                    Err(e) => println!("Error: {}", e),
                                }
                                terminal::enable_raw_mode()?;

                                render(&mut stdout, &repl)?;
                            },
                            ReplAction::AcceptLine { line: _, prompt: _ } => {
                                // User hit Enter for multi-line.
                                // Screen shows ">>> if True:". Cursor at end.
                                // We just need to move to next line.
                                stdout.execute(cursor::MoveToNextLine(1))?;
                                render(&mut stdout, &repl)?;
                            },
                            ReplAction::Render => {
                                render(&mut stdout, &repl)?;
                            },
                            ReplAction::ClearScreen => {
                                stdout.execute(terminal::Clear(ClearType::All))?;
                                stdout.execute(cursor::MoveTo(0, 0))?;
                                render(&mut stdout, &repl)?;
                            },
                            ReplAction::None => {},
                        }
                    }
                }
                _ => {}
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
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::ClearScreen),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::EOF),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::KillLine),
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::KillToEnd),
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::WordBackspace),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Input::HistorySearch),
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

    // We are on the "current line".
    // Clear it first.
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;
    stdout.queue(cursor::MoveToColumn(0))?;

    let full_line = format!("{}{}", state.prompt.as_str().blue(), state.buffer);
    stdout.write_all(full_line.as_bytes())?;

    let cursor_col = state.prompt.len() as u16 + state.cursor as u16;
    stdout.queue(cursor::MoveToColumn(cursor_col))?;

    stdout.flush()?;
    Ok(())
}
