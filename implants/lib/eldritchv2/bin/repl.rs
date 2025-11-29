use eldritchv2::lexer::Lexer;
use eldritchv2::token::Token;
use eldritchv2::{Interpreter, Value};
use rustyline::error::ReadlineError;
use rustyline::{Cmd, Config, EditMode, Editor, KeyCode, KeyEvent, Modifiers};

fn main() -> rustyline::Result<()> {
    let mut interpreter = Interpreter::new();

    // Register STD-dependent builtins
    interpreter.register_function("print", |args| {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{}", arg.to_string());
        }
        println!();
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
    println!("Tab key inserts 4 spaces.");
    println!("Built-ins: dir() can be used to inspect object attributes.");

    // Explicitly configure Emacs mode to ensure standard control keys work
    let config = Config::builder().edit_mode(EditMode::Emacs).build();

    // Initialize Rustyline editor with the config
    let mut rl = Editor::<(), rustyline::history::FileHistory>::with_config(config)?;

    // Bind Tab to insert 4 spaces instead of triggering completion
    rl.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        Cmd::Insert(1, "    ".into()),
    );

    if rl.load_history("eldritch_history.txt").is_err() {
        // No history file found, start fresh
    }

    let mut buffer = String::new();

    loop {
        // Decide prompt based on whether we're in a block
        let prompt = if buffer.is_empty() { ">>> " } else { "... " };

        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());

                let trimmed_line = line.trim();
                if trimmed_line == "exit" {
                    break;
                }

                buffer.push_str(&line);
                buffer.push('\n');

                // Check for incomplete input (open brackets or strings)
                let mut is_incomplete = false;
                let lex_result = Lexer::new(buffer.clone()).scan_tokens();

                match lex_result {
                    Ok(tokens) => {
                        // Check for unbalanced nesting
                        let mut balance = 0;
                        for t in tokens {
                            match t {
                                Token::LParen | Token::LBracket | Token::LBrace => balance += 1,
                                Token::RParen | Token::RBracket | Token::RBrace => {
                                    if balance > 0 {
                                        balance -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if balance > 0 {
                            is_incomplete = true;
                        }
                    }
                    Err(e) => {
                        // Check for unterminated strings that allow newlines (triple quoted)
                        // The lexer error "Unterminated string literal on line X" implies EOF was reached inside string.
                        // If it says "(newline)", it's a single-line string error, which we don't wait for.
                        if e.contains("Unterminated string literal") && !e.contains("(newline)") {
                            is_incomplete = true;
                        }
                    }
                }

                if is_incomplete {
                    continue;
                }

                // Heuristic:
                // 1. If the line ends with ':', it's definitely a block start (if/for/def).
                // 2. If we are already inside a block (buffer has multiple lines), we wait for an empty line.
                let ends_with_colon = trimmed_line.ends_with(':');
                let is_multi_line = buffer.lines().count() > 1;
                let is_empty_input = trimmed_line.is_empty();

                // Execute if:
                // - It's a single line that DOESN'T end in ':' (e.g., print(x) or d = {'a':1})
                // - OR we are in a multi-line block and the user hit Enter on an empty line.
                let should_execute =
                    (!ends_with_colon && !is_multi_line) || (is_multi_line && is_empty_input);

                if should_execute {
                    match interpreter.interpret(&buffer) {
                        Ok(v) => {
                            if !matches!(v, Value::None) {
                                println!("{:?}", v)
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                    buffer.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                buffer.clear();
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

    rl.save_history("eldritch_history.txt")?;
    Ok(())
}
