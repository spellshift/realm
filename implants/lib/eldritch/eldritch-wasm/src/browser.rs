use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::{BufferPrinter, Interpreter, Lexer, TokenKind};
use wasm_bindgen::prelude::*;

#[cfg(feature = "fake_bindings")]
use eldritch::{
    agent::fake::AgentLibraryFake, assets::fake::FakeAssetsLibrary,
    crypto::fake::CryptoLibraryFake, file::fake::FileLibraryFake, http::fake::HttpLibraryFake,
    pivot::fake::PivotLibraryFake, process::fake::ProcessLibraryFake,
    random::fake::RandomLibraryFake, regex::fake::RegexLibraryFake,
    report::fake::ReportLibraryFake, sys::fake::SysLibraryFake, time::fake::TimeLibraryFake,
};

#[wasm_bindgen]
pub struct BrowserRepl {
    buffer: String,
    interpreter: Interpreter,
}

#[wasm_bindgen]
impl BrowserRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BrowserRepl {
        let printer = Arc::new(BufferPrinter::new());
        let mut interp = Interpreter::new_with_printer(printer);

        #[cfg(feature = "fake_bindings")]
        {
            interp.register_lib(FileLibraryFake::default());
            interp.register_lib(ProcessLibraryFake::default());
            interp.register_lib(SysLibraryFake::default());
            interp.register_lib(HttpLibraryFake::default());
            interp.register_lib(CryptoLibraryFake::default());
            interp.register_lib(AgentLibraryFake::default());
            interp.register_lib(FakeAssetsLibrary::default());
            interp.register_lib(PivotLibraryFake::default());
            interp.register_lib(RandomLibraryFake::default());
            interp.register_lib(RegexLibraryFake::default());
            interp.register_lib(ReportLibraryFake::default());
            interp.register_lib(TimeLibraryFake::default());
        }

        BrowserRepl {
            buffer: String::new(),
            interpreter: interp,
        }
    }

    pub fn input(&mut self, line: &str) -> String {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        self.buffer = expand_macros(&self.buffer);

        let trimmed = self.buffer.trim();
        if trimmed == "exit" {
            let payload = self.buffer.clone();
            self.buffer.clear();
            return format!("{{ \"status\": \"complete\", \"payload\": {:?} }}", payload);
        }

        // Check for completeness
        let mut balance = 0;
        let mut is_incomplete_string = false;
        let mut has_error = false;
        let mut error_msg = String::new();

        let tokens = Lexer::new(self.buffer.clone()).scan_tokens();
        for t in tokens {
            match t.kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => balance += 1,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    if balance > 0 {
                        balance -= 1;
                    }
                }
                TokenKind::Error(ref msg) => {
                    if msg.contains("Unterminated string literal") && !msg.contains("(newline)") {
                        is_incomplete_string = true;
                    } else {
                        // Genuine error
                        has_error = true;
                        error_msg = msg.clone();
                    }
                }
                _ => {}
            }
        }

        // If we have an open bracket/paren/brace or incomplete string, it's definitely incomplete.
        if balance > 0 || is_incomplete_string {
            return String::from("{ \"status\": \"incomplete\", \"prompt\": \".. \" }");
        }

        // If there's a syntax error that isn't just "incomplete string", it might be a real error,
        // OR it might be incomplete code that looks like an error (e.g. `def foo`).
        // However, `eldritch-repl` logic is: if balance > 0 || incomplete_string -> incomplete.
        // Otherwise, check for colon at end of line or if it's a single line.

        if has_error {
            self.buffer.clear();
            return format!("{{ \"status\": \"error\", \"message\": {:?} }}", error_msg);
        }

        let ends_with_colon = trimmed.ends_with(':');
        let lines: Vec<&str> = self.buffer.lines().collect();
        let line_count = lines.len();

        let is_complete = if line_count == 1 && !ends_with_colon {
            true
        } else if (line_count > 1 || ends_with_colon) && line.trim().is_empty() {
            true
        } else {
            false
        };

        if is_complete {
            let payload = self.buffer.clone();
            self.buffer.clear();

            // Check for meta function
            let parsed_tokens = Lexer::new(payload.clone()).scan_tokens();
            let mut parser = eldritch_core::Parser::new(parsed_tokens);
            let (stmts, errs) = parser.parse();
            if errs.is_empty() && stmts.len() == 1 {
                if let eldritch_core::StmtKind::Expression(expr) = &stmts[0].kind {
                    if let eldritch_core::ExprKind::Call(callee, args) = &expr.kind {
                        if let eldritch_core::ExprKind::Identifier(name) = &callee.kind {
                            if name == "ssh" && args.len() == 1 {
                                if let eldritch_core::Argument::Positional(arg_expr) = &args[0] {
                                    if let eldritch_core::ExprKind::Literal(eldritch_core::Value::String(val)) = &arg_expr.kind {
                                        return format!("{{ \"status\": \"meta\", \"function\": \"ssh\", \"arguments\": [{:?}] }}", val);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return format!("{{ \"status\": \"complete\", \"payload\": {:?} }}", payload);
        }

        // Otherwise incomplete
        String::from("{ \"status\": \"incomplete\", \"prompt\": \".. \" }")
    }

    pub fn complete(&self, line: &str, cursor: usize) -> String {
        let (start, candidates) = self.interpreter.complete(line, cursor);

        let mut json = String::from("{ \"suggestions\": [");
        for (i, c) in candidates.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!("{:?}", c));
        }
        json.push_str(&format!("], \"start\": {} }}", start));
        json
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

fn expand_macros(code: &str) -> String {
    let mut expanded_code = code.to_string();

    loop {
        let tokens = Lexer::new(expanded_code.clone()).scan_tokens();
        let first_error = tokens.iter().find_map(|t| match &t.kind {
            TokenKind::Error(msg) => Some(msg.clone()),
            _ => None,
        });

        if let Some(msg) = first_error {
            if let Some(line_num_str) = msg.strip_prefix("Unexpected character: ! on line ") {
                let line_num: usize = match line_num_str.trim().parse() {
                    Ok(n) => n,
                    Err(_) => break,
                };

                if line_num == 0 {
                    break;
                }

                let lines: Vec<&str> = expanded_code.lines().collect();
                if line_num > lines.len() {
                    break;
                }

                let line_idx = line_num - 1;
                let line = lines[line_idx];

                let trimmed_line = line.trim_start();
                if let Some(rest) = trimmed_line.strip_prefix('!') {
                    let indentation = &line[..line.len() - trimmed_line.len()];

                    let cmd = rest;
                    let escaped_cmd = cmd.replace('\\', "\\\\").replace('"', "\\\"");
                    let macro_var = "_nonomacroclowntown";
                    let replacement = alloc::format!(
                        "{indentation}for {macro_var} in range(1):\n{indentation}\t{macro_var} = sys.shell(\"{escaped_cmd}\")\n{indentation}\tprint({macro_var}['stdout']);print({macro_var}['stderr'])"
                    );

                    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                    new_lines[line_idx] = replacement;

                    expanded_code = new_lines.join("\n");

                    if code.ends_with('\n') && !expanded_code.ends_with('\n') {
                        expanded_code.push('\n');
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    expanded_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_repl_simple() {
        let mut repl = BrowserRepl::new();
        let res = repl.input("print('hello')");
        assert!(res.contains("\"status\": \"complete\""));
        assert!(res.contains("\"payload\": \"print('hello')\""));
    }

    #[test]
    fn test_browser_repl_incomplete() {
        let mut repl = BrowserRepl::new();
        let res = repl.input("def foo():");
        assert!(res.contains("\"status\": \"incomplete\""));

        let res = repl.input("  pass");
        assert!(res.contains("\"status\": \"incomplete\""));

        let res = repl.input("");
        assert!(res.contains("\"status\": \"complete\""));
        assert!(res.contains("def foo():"));
        assert!(res.contains("pass"));
    }

    #[test]
    fn test_browser_repl_complete() {
        let repl = BrowserRepl::new();
        let res = repl.complete("pri", 3);
        assert!(res.contains("\"suggestions\":"));
        assert!(res.contains("print"));
        assert!(res.contains("\"start\":"));
    }

    #[test]
    fn test_browser_repl_reset() {
        let mut repl = BrowserRepl::new();
        let res = repl.input("def foo():");
        assert!(res.contains("\"status\": \"incomplete\""));
        repl.reset();
        let res = repl.input("print('reset')");
        assert!(res.contains("\"status\": \"complete\""));
        assert!(res.contains("print('reset')"));
    }

    #[test]
    fn test_browser_repl_macro() {
        let mut repl = BrowserRepl::new();
        repl.input("!ls");
        let res = repl.input("");
        assert!(res.contains("sys.shell"));
        assert!(res.contains("ls"));
    }

    #[test]
    fn test_browser_repl_macro_indent() {
        let mut repl = BrowserRepl::new();
        let _ = repl.input("def foo():");
        let res = repl.input("    !ls");
        assert!(res.contains("\"status\": \"incomplete\""));
        let res = repl.input("");
        assert!(res.contains("\"status\": \"complete\""));
        assert!(res.contains("sys.shell(\\\"ls\\\")"));
    }

    #[test]
    fn test_meta_ssh() {
        let mut repl = BrowserRepl::new();
        let res = repl.input("ssh('user:pass@host:22')");
        println!("{}", res);
        assert!(res.contains("\"status\": \"meta\""));
    }
}
