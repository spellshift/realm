use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::lexer::Lexer;
use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Char(char),
    Enter,
    ForceEnter, // Shift+Enter
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
    KillToEnd,       // Ctrl+K
    KillLine,        // Ctrl+U
    WordBackspace,   // Ctrl+W
    ClearScreen,     // Ctrl+L
    Cancel,          // Ctrl+C
    EOF,             // Ctrl+D
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplAction {
    None,
    Render, // State changed, need redraw
    Submit { code: String, last_line: String, prompt: String }, // Command block ready to execute
    AcceptLine { line: String, prompt: String }, // Intermediate line accepted (for multi-line)
    Quit,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub prompt: String,
    pub buffer: String,
    pub cursor: usize,
}

pub struct Repl {
    // Current editing line
    buffer: String,
    cursor: usize,

    // Multi-line accumulation
    // "Committed" lines of the current block
    pending_block: String,

    // History
    history: Vec<String>,
    history_idx: Option<usize>, // None = editing new line
    saved_buffer: String, // Buffer content before history navigation

    // State
    is_multiline: bool,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            buffer: String::new(),
            cursor: 0,
            pending_block: String::new(),
            history: Vec::new(),
            history_idx: None,
            saved_buffer: String::new(),
            is_multiline: false,
        }
    }

    pub fn load_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    fn current_prompt(&self) -> String {
        if self.pending_block.is_empty() { ">>> ".to_string() } else { "... ".to_string() }
    }

    pub fn get_render_state(&self) -> RenderState {
        RenderState {
            prompt: self.current_prompt(),
            buffer: self.buffer.clone(),
            cursor: self.cursor,
        }
    }

    pub fn handle_input(&mut self, input: Input) -> ReplAction {
        match input {
            Input::Char(c) => self.insert_char(c),
            Input::Tab => self.insert_str("    "),
            Input::Enter => self.handle_enter(false),
            Input::ForceEnter => self.handle_enter(true),
            Input::Backspace => self.backspace(),
            Input::Delete => self.delete(),
            Input::Left => self.move_left(),
            Input::Right => self.move_right(),
            Input::Up => self.history_up(),
            Input::Down => self.history_down(),
            Input::Home => self.move_home(),
            Input::End => self.move_end(),
            Input::KillToEnd => self.kill_to_end(),
            Input::KillLine => self.kill_line(),
            Input::WordBackspace => self.word_backspace(),
            Input::ClearScreen => ReplAction::Render,
            Input::Cancel => self.cancel(),
            Input::EOF => ReplAction::Quit,
        }
    }

    fn insert_char(&mut self, c: char) -> ReplAction {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
        ReplAction::Render
    }

    fn insert_str(&mut self, s: &str) -> ReplAction {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
        ReplAction::Render
    }

    fn backspace(&mut self) -> ReplAction {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn delete(&mut self) -> ReplAction {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn move_left(&mut self) -> ReplAction {
        if self.cursor > 0 {
            self.cursor -= 1;
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn move_right(&mut self) -> ReplAction {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn move_home(&mut self) -> ReplAction {
        self.cursor = 0;
        ReplAction::Render
    }

    fn move_end(&mut self) -> ReplAction {
        self.cursor = self.buffer.len();
        ReplAction::Render
    }

    fn kill_to_end(&mut self) -> ReplAction {
        if self.cursor < self.buffer.len() {
            self.buffer.truncate(self.cursor);
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn kill_line(&mut self) -> ReplAction {
        self.buffer.clear();
        self.cursor = 0;
        ReplAction::Render
    }

    fn word_backspace(&mut self) -> ReplAction {
        if self.cursor == 0 {
            return ReplAction::None;
        }
        let mut idx = self.cursor;
        let chars: Vec<char> = self.buffer.chars().collect();

        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }

        self.buffer.replace_range(idx..self.cursor, "");
        self.cursor = idx;
        ReplAction::Render
    }

    fn history_up(&mut self) -> ReplAction {
        if self.history.is_empty() {
            return ReplAction::None;
        }

        if self.history_idx.is_none() {
            self.saved_buffer = self.buffer.clone();
            self.history_idx = Some(self.history.len() - 1);
        } else {
            let idx = self.history_idx.unwrap();
            if idx > 0 {
                self.history_idx = Some(idx - 1);
            } else {
                return ReplAction::None;
            }
        }

        if let Some(idx) = self.history_idx {
            self.buffer = self.history[idx].clone();
            self.cursor = self.buffer.len();
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn history_down(&mut self) -> ReplAction {
        if self.history_idx.is_none() {
            return ReplAction::None;
        }

        let idx = self.history_idx.unwrap();
        if idx + 1 < self.history.len() {
            self.history_idx = Some(idx + 1);
            self.buffer = self.history[self.history_idx.unwrap()].clone();
            self.cursor = self.buffer.len();
        } else {
            self.history_idx = None;
            self.buffer = self.saved_buffer.clone();
            self.cursor = self.buffer.len();
        }
        ReplAction::Render
    }

    fn cancel(&mut self) -> ReplAction {
        self.buffer.clear();
        self.cursor = 0;
        self.pending_block.clear();
        self.is_multiline = false;
        self.history_idx = None;
        ReplAction::Render
    }

    fn handle_enter(&mut self, force: bool) -> ReplAction {
        // Capture state before modification
        let current_prompt = self.current_prompt();
        let last_line = self.buffer.clone();

        let mut full_code = self.pending_block.clone();
        if !self.pending_block.is_empty() {
            full_code.push('\n');
        }
        full_code.push_str(&self.buffer);

        let should_submit = if force {
            false
        } else {
            self.should_execute(&full_code, &self.buffer)
        };

        if should_submit {
            if !full_code.trim().is_empty() {
                if self.history.last() != Some(&full_code) {
                    self.history.push(full_code.clone());
                }
            }

            self.buffer.clear();
            self.cursor = 0;
            self.pending_block.clear();
            self.history_idx = None;
            self.is_multiline = false;

            ReplAction::Submit {
                code: full_code,
                last_line,
                prompt: current_prompt
            }
        } else {
            self.pending_block = full_code;
            self.buffer.clear();
            self.cursor = 0;
            self.history_idx = None;
            self.is_multiline = true;

            ReplAction::AcceptLine {
                line: last_line,
                prompt: current_prompt
            }
        }
    }

    fn should_execute(&self, full_code: &str, last_line: &str) -> bool {
        let trimmed_last = last_line.trim();
        let trimmed_code = full_code.trim();

        if trimmed_code == "exit" {
            return true;
        }

        let mut balance = 0;
        let mut is_incomplete_string = false;

        match Lexer::new(full_code.to_string()).scan_tokens() {
            Ok(tokens) => {
                for t in tokens {
                    match t.kind {
                        TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => balance += 1,
                        TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                            if balance > 0 { balance -= 1; }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                 if e.contains("Unterminated string literal") && !e.contains("(newline)") {
                     is_incomplete_string = true;
                 }
            }
        }

        if balance > 0 || is_incomplete_string {
            return false;
        }

        let ends_with_colon = trimmed_last.ends_with(':');
        let is_empty_last = trimmed_last.is_empty();
        let line_count = full_code.lines().count();

        if line_count == 1 && !ends_with_colon {
            return true;
        }

        if line_count > 1 && is_empty_last {
            return true;
        }

        false
    }
}
