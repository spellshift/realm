use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::lang::lexer::Lexer;
use crate::lang::token::TokenKind;

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
    HistorySearch,   // Ctrl+R
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplAction {
    None,
    Render, // State changed, need redraw
    ClearScreen, // Clear screen request
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

#[derive(Debug, Clone)]
struct SearchState {
    query: String,
    match_index: Option<usize>, // Index in history
    saved_buffer: String,       // Buffer content before search started
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
    search_state: Option<SearchState>,
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
            search_state: None,
        }
    }

    pub fn load_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    fn current_prompt(&self) -> String {
        if let Some(ref search) = self.search_state {
            return alloc::format!("(reverse-i-search)`{}': ", search.query);
        }
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
        if self.search_state.is_some() {
            return self.handle_search_input(input);
        }

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
            Input::ClearScreen => ReplAction::ClearScreen,
            Input::Cancel => self.cancel(),
            Input::EOF => ReplAction::Quit,
            Input::HistorySearch => self.start_search(),
        }
    }

    fn start_search(&mut self) -> ReplAction {
        if self.history.is_empty() {
            return ReplAction::None;
        }
        self.search_state = Some(SearchState {
            query: String::new(),
            match_index: None,
            saved_buffer: self.buffer.clone(),
        });
        self.buffer.clear();
        self.cursor = 0;
        ReplAction::Render
    }

    fn handle_search_input(&mut self, input: Input) -> ReplAction {
        match input {
            Input::HistorySearch => self.search_next(),
            Input::Char(c) => self.search_append(c),
            Input::Backspace => self.search_backspace(),
            Input::Enter => self.end_search(true),
            Input::Cancel => self.end_search(false), // Cancel restores original buffer
            // Navigation keys could accept the search and move cursor?
            Input::Left | Input::Right | Input::Home | Input::End => {
                self.end_search(true);
                // Re-process the navigation input on the restored buffer?
                // For simplicity, just accept the search result and let user navigate next.
                // Or we could re-dispatch. Let's just accept.
                ReplAction::Render
            },
            _ => ReplAction::None,
        }
    }

    fn search_next(&mut self) -> ReplAction {
        // Find next match backwards
        let state = self.search_state.as_ref().unwrap();
        if state.query.is_empty() {
             return ReplAction::None;
        }

        let start_idx = state.match_index.unwrap_or(self.history.len());
        if start_idx == 0 {
            return ReplAction::None;
        }

        let query = state.query.clone();
        // Search backwards from start_idx - 1
        for i in (0..start_idx).rev() {
            if self.history[i].contains(&query) {
                self.search_state.as_mut().unwrap().match_index = Some(i);
                self.buffer = self.history[i].clone();
                // Cursor position: Zsh usually puts it at end of match or end of line.
                // We'll put it at end of line to visualize the match clearly.
                // Wait, in search mode, the buffer displayed IS the match.
                // The prompt shows the query.

                // Let's highlight match?
                // RenderState only has one buffer string.
                // We just show the history line in the buffer.
                // The cursor in RenderState is an index into buffer.
                // We can set cursor to where the match starts?
                if let Some(pos) = self.buffer.find(&query) {
                    self.cursor = pos; // Point to start of match?
                } else {
                    self.cursor = 0;
                }
                return ReplAction::Render;
            }
        }

        ReplAction::None
    }

    fn search_append(&mut self, c: char) -> ReplAction {
        if let Some(state) = self.search_state.as_mut() {
            state.query.push(c);
            state.match_index = None; // Reset match index to search from end
        }
        // Trigger a search with new query
        self.perform_search()
    }

    fn search_backspace(&mut self) -> ReplAction {
        if let Some(state) = self.search_state.as_mut() {
            if !state.query.is_empty() {
                state.query.pop();
                state.match_index = None;
            }
        }
        self.perform_search()
    }

    fn perform_search(&mut self) -> ReplAction {
        let query = self.search_state.as_ref().unwrap().query.clone();
        if query.is_empty() {
            self.buffer.clear();
            self.cursor = 0;
            return ReplAction::Render;
        }

        // Always search from end for fresh query
        for (i, item) in self.history.iter().enumerate().rev() {
            if item.contains(&query) {
                self.search_state.as_mut().unwrap().match_index = Some(i);
                self.buffer = item.clone();
                if let Some(pos) = self.buffer.find(&query) {
                    self.cursor = pos;
                } else {
                    self.cursor = 0;
                }
                return ReplAction::Render;
            }
        }

        // No match
        self.buffer.clear(); // Or keep previous match? Standard is usually showing failing search
        // We'll clear for now to indicate no match found
        self.cursor = 0;
        ReplAction::Render
    }

    fn end_search(&mut self, accept: bool) -> ReplAction {
        let saved = self.search_state.as_ref().unwrap().saved_buffer.clone();

        if accept {
             // Keep current buffer (the match)
             // Restore saved buffer if no match was found (buffer empty)?
             // If buffer is empty (no match), maybe restore saved.
             if self.buffer.is_empty() {
                 self.buffer = saved;
             }
        } else {
            // Restore original buffer
            self.buffer = saved;
        }

        self.cursor = self.buffer.len(); // Move cursor to end
        self.search_state = None;
        ReplAction::Render
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
