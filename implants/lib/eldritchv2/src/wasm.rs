use wasm_bindgen::prelude::*;
use crate::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::token::TokenKind;
use alloc::string::ToString;
use alloc::string::String;
use alloc::format;

#[wasm_bindgen]
extern "C" {
    fn repl_print(s: &str);
}

fn wasm_print(args: &[Value]) -> Result<Value, String> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }
    repl_print(&out);
    Ok(Value::None)
}

#[wasm_bindgen]
pub struct WasmInterpreter {
    interp: Interpreter,
}

#[wasm_bindgen]
impl WasmInterpreter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmInterpreter {
        let mut interp = Interpreter::new();
        interp.register_function("print", wasm_print);
        WasmInterpreter { interp }
    }

    pub fn run(&mut self, code: &str) -> String {
        match self.interp.interpret(code) {
            Ok(v) => {
                if let Value::None = v {
                    String::new()
                } else {
                    v.to_string()
                }
            },
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[wasm_bindgen]
pub fn check_complete(code: &str) -> bool {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Lexer check for balance and string termination
    let lex_result = Lexer::new(code.to_string()).scan_tokens();
    let is_syntactically_complete = match lex_result {
        Ok(tokens) => {
            let mut balance = 0;
            for t in tokens {
                match t.kind {
                    TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => balance += 1,
                    TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                        if balance > 0 { balance -= 1; }
                    }
                    _ => {}
                }
            }
            balance == 0
        }
        Err(e) => {
            // If "Unterminated string literal" and NOT "(newline)", it's incomplete string
            if e.contains("Unterminated string literal") && !e.contains("(newline)") {
                false
            } else {
                true // Syntax error or complete string
            }
        }
    };

    if !is_syntactically_complete {
        return false;
    }

    // Heuristics
    let ends_with_colon = trimmed.ends_with(':');
    let is_multi_line = trimmed.lines().count() > 1;

    // Check for empty line at the end (indicates block termination)
    // We look for two newlines at the end, possibly separated by whitespace.
    // Since we expect the JS to append \n after every input, a blank line input results in \n\s*\n at the end.
    // We need to be careful with \r\n vs \n.
    // Let's check if the suffix matches \n\s*\n?
    // Or just look at the last characters.
    let ends_with_empty_line = {
        // Find last newline
        if let Some(last_nl) = code.rfind('\n') {
             // Check content between prev newline and this one
             if let Some(prev_nl) = code[..last_nl].rfind('\n') {
                 let last_line = &code[prev_nl+1..last_nl];
                 last_line.trim().is_empty()
             } else {
                 // Only one newline found.
                 // If the string starts with empty line?
                 // e.g. "   \n"
                 code[..last_nl].trim().is_empty()
             }
        } else {
            false
        }
    };

    // Execute if:
    // 1. Single line AND not ending in colon.
    // 2. Multi-line AND ends with empty line.
    if !is_multi_line && !ends_with_colon {
        true
    } else if is_multi_line && ends_with_empty_line {
        true
    } else {
        false
    }
}
