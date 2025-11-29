use eldritchv2::{Interpreter, Value};
use rustyline::error::ReadlineError;
use rustyline::{Config, EditMode, Editor};

fn main() -> rustyline::Result<()> {
    let mut interpreter = Interpreter::new();

    // Register STD-dependent builtins
    interpreter.register_function("print", |args| {
        println!("{}", args[0].to_string());
        Ok(Value::None)
    });

    interpreter.register_function("input", |_| {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => Ok(Value::String(input.trim().to_string())),
            Err(e) => Err(format!("Input error: {}", e)),
        }
    });

    println!("Starlark-like REPL (Rust)");
    println!("Type 'exit' to quit. End blocks with an empty line.");
    println!("Features: Arrow keys, History (Ctrl+R), Clear (Ctrl+L).");
    println!(
        "Navigation: Start (Ctrl+A), End (Ctrl+E). Editing: Del Word (Ctrl+W), Del Line (Ctrl+U)."
    );

    // Explicitly configure Emacs mode to ensure standard control keys work
    let config = Config::builder().edit_mode(EditMode::Emacs).build();

    // Initialize Rustyline editor with the config
    // DefaultEditor is an alias for Editor<(), FileHistory>, so we construct it explicitly to pass config
    let mut rl = Editor::<(), rustyline::history::FileHistory>::with_config(config)?;

    if rl.load_history("history.txt").is_err() {
        // No history file found, start fresh
    }

    let mut buffer = String::new();

    loop {
        // Decide prompt based on whether we're in a block
        let prompt = if buffer.is_empty() { ">>> " } else { "... " };

        // readline handles the raw mode and control characters (Arrows, Ctrl+R, Ctrl+L, etc.)
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                // Add valid lines to history
                let _ = rl.add_history_entry(line.as_str());

                let trimmed_line = line.trim();
                if trimmed_line == "exit" {
                    break;
                }

                // Accumulate buffer (Rustyline strips the newline, so we add it back)
                buffer.push_str(&line);
                buffer.push('\n');

                // Heuristic: Execute if it's a single line without control flow, or if we hit an empty line after a block started.
                let has_control_flow = buffer.contains("def")
                    || buffer.contains("if")
                    || buffer.contains("else")
                    || buffer.contains("for");
                let is_empty_input = trimmed_line.is_empty();

                // Check if we have a complete statement or simple expression
                let is_single_line_expression =
                    !buffer.contains(':') && buffer.lines().count() <= 1;

                if is_single_line_expression || (is_empty_input && has_control_flow) {
                    // Execute the accumulated buffer
                    match interpreter.interpret(&buffer) {
                        Ok(v) => {
                            if !matches!(v, Value::None) {
                                println!("{:?}", v)
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                    buffer.clear();
                } else if is_empty_input {
                    // Clear buffer if the user hits enter on a blank line outside of an active control flow structure
                    if !has_control_flow {
                        buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                buffer.clear(); // Reset buffer on Ctrl+C
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history on exit
    rl.save_history("history.txt")?;
    Ok(())
}
