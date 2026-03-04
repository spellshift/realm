use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::{BufferPrinter, Interpreter, Lexer, TokenKind};
use eldritch_repl::{Input, Repl, ReplAction};
use wasm_bindgen::prelude::*;

#[cfg(feature = "fake_bindings")]
use eldritch::{
    agent::fake::AgentLibraryFake, assets::fake::FakeAssetsLibrary,
    crypto::fake::CryptoLibraryFake, file::fake::FileLibraryFake, http::fake::HttpLibraryFake,
    pivot::fake::PivotLibraryFake, process::fake::ProcessLibraryFake,
    random::fake::RandomLibraryFake, regex::fake::RegexLibraryFake,
    report::fake::ReportLibraryFake, sys::fake::SysLibraryFake, time::fake::TimeLibraryFake,
};

#[wasm_bindgen(getter_with_clone)]
pub struct ReplState {
    pub status: String,
    pub prompt: String,
    pub buffer: String,
    pub cursor_pos: usize,
    pub payload: Option<String>,
    pub function: Option<String>,
    pub args: Option<Vec<String>>,
    pub is_running: bool,
}

impl Default for ReplState {
    fn default() -> Self {
        Self {
            status: String::from("render"),
            prompt: String::from(">>> "),
            buffer: String::new(),
            cursor_pos: 0,
            payload: None,
            function: None,
            args: None,
            is_running: false,
        }
    }
}

#[wasm_bindgen]
pub struct BrowserRepl {
    repl: Repl,
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
            repl: Repl::new(),
            interpreter: interp,
        }
    }

    pub fn handle_key(
        &mut self,
        key: &str,
        ctrl: bool,
        alt: bool,
        meta: bool,
        shift: bool,
    ) -> ReplState {
        // Map DOM key to Input
        let input = match key {
            "Enter" => {
                if shift {
                    Input::ForceEnter
                } else {
                    Input::Enter
                }
            }
            "Backspace" => {
                if alt || ctrl {
                    Input::WordBackspace
                } else {
                    Input::Backspace
                }
            }
            "Delete" => Input::Delete,
            "ArrowLeft" => Input::Left,
            "ArrowRight" => Input::Right,
            "ArrowUp" => Input::Up,
            "ArrowDown" => Input::Down,
            "Home" => Input::Home,
            "End" => Input::End,
            "Tab" => Input::Tab,
            "Escape" => Input::Cancel,
            "a" if ctrl => Input::Home,
            "e" if ctrl => Input::End,
            "k" if ctrl => Input::KillToEnd,
            "u" if ctrl => Input::KillLine,
            "l" if ctrl => Input::ClearScreen,
            "c" if ctrl => Input::Cancel,
            "d" if ctrl => Input::EOF,
            "r" if ctrl => Input::HistorySearch,
            " " if ctrl => Input::ForceComplete,
            "b" if alt => Input::Left,
            "f" if alt => Input::Right,
            "d" if alt => Input::WordBackspace, // Note: alt+d is forward delete in some shells, mapping to word backspace for simplicity or you can change
            _ if key.chars().count() == 1 && !ctrl && !meta => {
                Input::Char(key.chars().next().unwrap())
            }
            _ => return self.build_state("render", None, None, None), // Unhandled key
        };

        let action = self.repl.handle_input(input);

        match action {
            ReplAction::None => self.build_state("render", None, None, None),
            ReplAction::Render => self.build_state("render", None, None, None),
            ReplAction::ClearScreen => {
                self.build_state("clear", None, None, None)
            }
            ReplAction::Complete => {
                let state = self.repl.get_render_state();
                let (start, candidates) = self.interpreter.complete(&state.buffer, state.cursor);
                self.repl.set_suggestions(candidates, start);
                self.build_state("render", None, None, None)
            }
            ReplAction::Submit { code, last_line: _, prompt: _ } => {
                // Parse code for meta functionalities
                // Only if the code is structured like `ssh("some-host")`
                if let Some((func, args)) = self.parse_meta_command(&code) {
                    if func == "ssh" {
                        return self.build_state("meta", None, Some(func), Some(args));
                    }
                }

                // If code is exit, we can return status exit, but browser currently checks payload == exit
                self.build_state("complete", Some(code), None, None)
            }
            ReplAction::AcceptLine { line: _, prompt: _ } => {
                self.build_state("render", None, None, None)
            }
            ReplAction::Quit => self.build_state("quit", None, None, None),
        }
    }

    fn build_state(&self, status: &str, payload: Option<String>, function: Option<String>, args: Option<Vec<String>>) -> ReplState {
        let rs = self.repl.get_render_state();
        ReplState {
            status: status.to_string(),
            prompt: rs.prompt.clone(),
            buffer: rs.buffer.clone(),
            cursor_pos: rs.cursor,
            payload,
            function,
            args,
            is_running: false, // Update as needed if we add running states
        }
    }

    fn parse_meta_command(&self, code: &str) -> Option<(String, Vec<String>)> {
        // Very basic manual parser for `ssh(...)` to avoid bringing in the full AST matching here if simple
        // Alternatively, use eldritch_core::Parser.
        let tokens = Lexer::new(code.to_string()).scan_tokens();
        
        let mut tokens_iter = tokens.iter();
        
        // ssh Token
        let first = tokens_iter.next()?;
        if let TokenKind::Identifier(id) = &first.kind {
            if id != "ssh" {
                return None;
            }
        } else {
            return None;
        }

        // Left Paren
        let second = tokens_iter.next()?;
        if !matches!(second.kind, TokenKind::LParen) {
            return None;
        }

        let mut args = Vec::new();
        let mut closed = false;

        loop {
            let t = tokens_iter.next();
            if t.is_none() {
                break;
            }
            let t = t.unwrap();
            
            match &t.kind {
                TokenKind::RParen => {
                    closed = true;
                    break;
                }
                TokenKind::String(s) => {
                    args.push(s.clone());
                }
                TokenKind::Comma => continue,
                _ => return None, // Only strings allowed for now
            }
        }

        if closed {
            // Check if there are trailing tokens other than EOF/Newline
            for t in tokens_iter {
                match &t.kind {
                    TokenKind::Eof | TokenKind::Newline => continue,
                    _ => return None,
                }
            }

            Some(("ssh".to_string(), args))
        } else {
            None
        }
    }

    pub fn input(&mut self, line: &str) -> String {
        // Legacy method, can just inject the characters then send enter
        for c in line.chars() {
            self.repl.handle_input(Input::Char(c));
        }
        let action = self.repl.handle_input(Input::Enter);
        
        match action {
            ReplAction::Submit { code, .. } => {
                format!("{{ \"status\": \"complete\", \"payload\": {:?} }}", code)
            }
            ReplAction::AcceptLine { prompt, .. } => {
                format!("{{ \"status\": \"incomplete\", \"prompt\": {:?} }}", prompt)
            }
            _ => String::from("{ \"status\": \"error\", \"message\": \"invalid state\" }"),
        }
    }

    pub fn get_suggestions(&mut self) -> Option<js_sys::Array> {
        let state = self.repl.get_render_state();
        if let Some(suggestions) = state.suggestions {
            let js_array = js_sys::Array::new();
            for s in suggestions {
                js_array.push(&JsValue::from_str(&s));
            }
            Some(js_array)
        } else {
            None
        }
    }

    pub fn get_suggestions_start(&self) -> Option<usize> {
        let state = self.repl.get_render_state();
        state.completion_start
    }

    pub fn get_suggestions_index(&self) -> Option<usize> {
        let state = self.repl.get_render_state();
        state.suggestion_idx
    }

    pub fn complete(&self, line: &str, cursor: usize) -> String {
        // We use the internal interpreter to get completions (Legacy JSON return)
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
        self.repl = Repl::new();
    }
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
