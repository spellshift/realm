// repl.rs

extern crate std;

use crate::evaluator::{EnvRef, Environment, EvalError, Evaluator, NativeFn, Value};
use crate::lexer::Lexer;
use crate::parser::Parser;
use alloc::string::String;
use std::io::{self, Write};

/// Start an interactive REPL for the DSL.
///
/// - `builtins` is a slice of (name, native_fn) to register in the initial environment.
/// - Use `:q` or `:quit` to exit.
pub fn start_repl(builtins: &[(&str, NativeFn)]) -> Result<(), EvalError> {
    // Create an environment with any built-in native functions.
    let env: EnvRef = Environment::with_builtins(builtins);
    let mut evaluator = Evaluator::with_env(env);

    std::println!("DSL REPL. Type :q or :quit to exit.");

    loop {
        // Prompt
        std::print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let n = match io::stdin().read_line(&mut input) {
            Ok(n) => n,
            Err(_) => break, // treat I/O error as exit
        };

        // EOF or empty line → exit
        if n == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Simple REPL commands
        if trimmed == ":q" || trimmed == ":quit" {
            break;
        }

        // Lex + parse a single line (or multi-line if user pasted)
        let lexer = Lexer::new(trimmed);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if !parser.errors().is_empty() {
            std::eprintln!("Parse errors:");
            for err in parser.errors() {
                std::eprintln!("  {}", err);
            }
            continue;
        }

        // Evaluate
        match evaluator.eval_program(&program) {
            Ok(Value::Null) => {
                // Print nothing for null results (like pure statements).
            }
            Ok(val) => {
                std::println!("{:?}", val);
            }
            Err(e) => {
                std::eprintln!("Evaluation error: {}", e.message());
            }
        }
    }

    Ok(())
}
