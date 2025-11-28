use eldritchv2::{Interpreter, Value};
use std::io::{self, Write};

fn main() {
    let mut interpreter = Interpreter::new();
    println!("Starlark-like REPL (Rust)");
    println!("Type 'exit' to quit. End blocks with an empty line.");

    let stdin = io::stdin();
    let mut buffer = String::new();

    loop {
        // Decide prompt based on whether we're in a block
        if buffer.is_empty() {
            print!(">>> ");
        } else {
            print!("... ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            continue;
        }

        let trimmed_line = line.trim();
        if trimmed_line == "exit" {
            break;
        }

        // Append line to buffer
        buffer.push_str(&line);

        // Heuristic: Execute if it's a single line without control flow, or if we hit an empty line after a block started.
        let has_control_flow =
            buffer.contains("def") || buffer.contains("if") || buffer.contains("else");
        let is_single_line_expression = !buffer.contains(':') && buffer.matches('\n').count() <= 1;

        if is_single_line_expression || (trimmed_line.is_empty() && has_control_flow) {
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
        } else if trimmed_line.is_empty() {
            // Clear buffer if the user hits enter on a blank line outside of an active control flow structure
            if !has_control_flow {
                buffer.clear();
            }
        }
    }
}
