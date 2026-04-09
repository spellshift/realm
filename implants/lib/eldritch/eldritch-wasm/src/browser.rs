use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::{BufferPrinter, ExprKind, Interpreter, Lexer, Parser, StmtKind, TokenKind};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MetaCommand {
    #[serde(rename = "help")]
    Help { target: Option<String> },
}

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
        for t in &tokens {
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

        // If we have a hard error from lexer (like bad char), we might return error.
        // But let's follow the REPL logic:
        // logic from repl:
        // if balance > 0 || is_incomplete_string -> false (incomplete)
        // ends_with_colon -> false (incomplete)
        // line_count == 1 && !ends_with_colon -> true (complete)
        // line_count > 1 && is_empty_last -> true (complete)

        if has_error {
            // If we have a lexer error that is NOT incomplete string, report error
            // Unless it's something that could be fixed by typing more?
            // Unexpected char is usually fatal.
            self.buffer.clear();
            return format!("{{ \"status\": \"error\", \"message\": {:?} }}", error_msg);
        }

        let ends_with_colon = trimmed.ends_with(':');
        let lines: Vec<&str> = self.buffer.lines().collect();
        let line_count = lines.len();
        let last_line_empty =
            self.buffer.ends_with('\n') && lines.last().map_or(true, |l| l.trim().is_empty());

        let mut is_complete = false;

        // If single line and doesn't end with colon, it's complete.
        if line_count == 1 && !ends_with_colon {
            is_complete = true;
        } else if (line_count > 1 || ends_with_colon) && line.trim().is_empty() {
            // If multi-line (or ends with colon), we need an empty line to finish.
            is_complete = true;
        }

        if is_complete {
            // Check for meta commands
            let mut meta_command = None;

            let mut parser = Parser::new(tokens);
            let (ast, errors) = parser.parse();

            if errors.is_empty() {
                if ast.len() == 1 {
                    if let StmtKind::Expression(expr) = &ast[0].kind {
                        match &expr.kind {
                            ExprKind::Identifier(id) if id == "help" => {
                                meta_command = Some(MetaCommand::Help { target: None });
                            }
                            ExprKind::Call(callee, args) => {
                                if let ExprKind::Identifier(id) = &callee.kind {
                                    if id == "help" {
                                        if args.is_empty() {
                                            meta_command = Some(MetaCommand::Help { target: None });
                                        } else if args.len() == 1 {
                                            if let eldritch_core::Argument::Positional(arg_expr) =
                                                &args[0]
                                            {
                                                // Try to format the argument back to string for the target
                                                let mut parts = Vec::new();
                                                let mut current_expr = arg_expr;
                                                let mut is_valid = true;

                                                while is_valid {
                                                    match &current_expr.kind {
                                                        ExprKind::Identifier(id) => {
                                                            parts.push(id.clone());
                                                            break;
                                                        }
                                                        ExprKind::GetAttr(obj, attr) => {
                                                            parts.push(attr.clone());
                                                            current_expr = obj;
                                                        }
                                                        _ => {
                                                            is_valid = false;
                                                            break;
                                                        }
                                                    }
                                                }

                                                let mut target_str = String::new();
                                                if is_valid {
                                                    parts.reverse();
                                                    target_str = parts.join(".");
                                                } else {
                                                    target_str = "unknown".to_string();
                                                }
                                                meta_command = Some(MetaCommand::Help {
                                                    target: Some(target_str),
                                                });
                                            } else {
                                                meta_command =
                                                    Some(MetaCommand::Help { target: None });
                                            }
                                        } else {
                                            meta_command = Some(MetaCommand::Help { target: None });
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if let Some(cmd) = meta_command {
                self.buffer.clear();
                if let Ok(json) = serde_json::to_string(&cmd) {
                    return format!("{{ \"status\": \"meta\", \"meta_command\": {} }}", json);
                }
            }

            let payload = self.buffer.clone();
            self.buffer.clear();
            return format!("{{ \"status\": \"complete\", \"payload\": {:?} }}", payload);
        }

        // Otherwise incomplete
        String::from("{ \"status\": \"incomplete\", \"prompt\": \".. \" }")
    }

    pub fn complete(&self, line: &str, cursor: usize) -> String {
        // We use the internal interpreter to get completions.
        // The interpreter has builtins loaded.
        let (start, candidates) = self.interpreter.complete(line, cursor);

        // Return JSON object with suggestions and start index
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
        // Payload check: depends on formatting, check substring
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
        // Start a block
        let _ = repl.input("def foo():");
        // Indented macro
        let res = repl.input("    !ls");

        // It should just append to buffer, so incomplete (inside block)
        assert!(res.contains("\"status\": \"incomplete\""));

        // Finish block
        let res = repl.input("");
        assert!(res.contains("\"status\": \"complete\""));
        // Payload should contain sys.shell with indentation
        assert!(res.contains("sys.shell(\\\"ls\\\")"));
    }
}
