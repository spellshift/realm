use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::{Lexer, TokenKind};

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
    ForceComplete, // Ctrl+Space
    KillToEnd,     // Ctrl+K
    KillLine,      // Ctrl+U
    WordBackspace, // Ctrl+W
    ClearScreen,   // Ctrl+L
    Cancel,        // Ctrl+C
    EOF,           // Ctrl+D
    HistorySearch, // Ctrl+R
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplAction {
    None,
    Render,      // State changed, need redraw
    ClearScreen, // Clear screen request
    Complete,    // Request completion
    Submit {
        code: String,
        last_line: String,
        prompt: String,
    }, // Command block ready to execute
    AcceptLine {
        line: String,
        prompt: String,
    }, // Intermediate line accepted (for multi-line)
    Quit,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub prompt: String,
    pub buffer: String,
    pub cursor: usize,
    pub suggestions: Option<Vec<String>>,
    pub suggestion_idx: Option<usize>,
    pub completion_start: Option<usize>,
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
    saved_buffer: String,       // Buffer content before history navigation

    // State
    is_multiline: bool,
    search_state: Option<SearchState>,
    suggestions: Option<Vec<String>>,
    suggestion_idx: Option<usize>,
    completion_start: Option<usize>,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
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
            suggestions: None,
            suggestion_idx: None,
            completion_start: None,
        }
    }

    pub fn load_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    pub fn set_suggestions(&mut self, suggestions: Vec<String>, start_index: usize) {
        if suggestions.is_empty() {
            self.suggestions = None;
            self.suggestion_idx = None;
            self.completion_start = None;
        } else {
            self.suggestions = Some(suggestions);
            self.suggestion_idx = Some(0);
            self.completion_start = Some(start_index);
        }
    }

    pub fn clear_suggestions(&mut self) {
        self.suggestions = None;
        self.suggestion_idx = None;
        self.completion_start = None;
    }

    fn current_prompt(&self) -> String {
        if let Some(ref search) = self.search_state {
            return alloc::format!("(reverse-i-search)`{}': ", search.query);
        }
        if self.pending_block.is_empty() {
            ">>> ".to_string()
        } else {
            "... ".to_string()
        }
    }

    pub fn get_render_state(&self) -> RenderState {
        // Calculate UTF-16 code units count for cursor position for JS compatibility
        // The `cursor` field tracks byte offset in UTF-8 buffer.
        // We also want to provide something that helps frontend align cursor.
        // But for now, we keep cursor as byte index, and we fix the REPL logic first.
        RenderState {
            prompt: self.current_prompt(),
            buffer: self.buffer.clone(),
            cursor: self.cursor,
            suggestions: self.suggestions.clone(),
            suggestion_idx: self.suggestion_idx,
            completion_start: self.completion_start,
        }
    }

    pub fn handle_input(&mut self, input: Input) -> ReplAction {
        if self.search_state.is_some() {
            return self.handle_search_input(input);
        }

        // Check for suggestion cycling
        if self.suggestions.is_some() {
            match input {
                Input::Down | Input::Tab => {
                    self.cycle_suggestion(1);
                    return ReplAction::Render;
                }
                Input::Up => {
                    self.cycle_suggestion(-1);
                    return ReplAction::Render;
                }
                Input::Enter => {
                    if let Some(idx) = self.suggestion_idx {
                        self.accept_suggestion(idx);
                        return ReplAction::Render;
                    }
                }
                Input::Cancel => {
                    self.clear_suggestions();
                    return ReplAction::Render;
                }
                _ => {
                    // Any other input clears suggestions and processes normally
                    self.clear_suggestions();
                }
            }
        }

        match input {
            Input::Char(c) => self.insert_char(c),
            Input::Tab => self.handle_tab(),
            Input::ForceComplete => ReplAction::Complete,
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

    fn cycle_suggestion(&mut self, direction: isize) {
        if let Some(suggestions) = &self.suggestions {
            let count = suggestions.len();
            if count == 0 {
                return;
            }
            let current = self.suggestion_idx.unwrap_or(0) as isize;
            let next = (current + direction).rem_euclid(count as isize);
            self.suggestion_idx = Some(next as usize);
        }
    }

    fn accept_suggestion(&mut self, idx: usize) {
        if let Some(suggestions) = &self.suggestions
            && idx < suggestions.len()
        {
            let suggestion = &suggestions[idx];
            if let Some(start) = self.completion_start {
                // Replace from start to cursor with suggestion
                if start <= self.cursor && start <= self.buffer.len() {
                    self.buffer.replace_range(start..self.cursor, suggestion);
                    self.cursor = start + suggestion.len();
                }
            }
        }
        self.clear_suggestions();
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
            Input::Cancel => self.end_search(false),
            Input::Left | Input::Right | Input::Home | Input::End => {
                self.end_search(true);
                ReplAction::Render
            }
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
                if let Some(pos) = self.buffer.find(&query) {
                    self.cursor = pos;
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
        if let Some(state) = self.search_state.as_mut()
            && !state.query.is_empty()
        {
            state.query.pop();
            state.match_index = None;
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
        self.buffer.clear();
        self.cursor = 0;
        ReplAction::Render
    }

    fn end_search(&mut self, accept: bool) -> ReplAction {
        let saved = self.search_state.as_ref().unwrap().saved_buffer.clone();

        if accept {
            if self.buffer.is_empty() {
                self.buffer = saved;
            }
        } else {
            self.buffer = saved;
        }

        self.cursor = self.buffer.len(); // Move cursor to end
        self.search_state = None;
        ReplAction::Render
    }

    fn handle_tab(&mut self) -> ReplAction {
        let line_up_to_cursor: String = self.buffer.chars().take(self.cursor).collect();
        if line_up_to_cursor.trim().is_empty() {
            self.insert_str("    ")
        } else {
            ReplAction::Complete
        }
    }

    fn insert_char(&mut self, c: char) -> ReplAction {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        ReplAction::Render
    }

    fn insert_str(&mut self, s: &str) -> ReplAction {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
        ReplAction::Render
    }

    fn backspace(&mut self) -> ReplAction {
        if self.cursor > 0 {
            // Traverse backwards from cursor
            if let Some(c) = self.buffer[..self.cursor].chars().next_back() {
                self.cursor -= c.len_utf8();
                self.buffer.remove(self.cursor);
                ReplAction::Render
            } else {
                ReplAction::None
            }
        } else {
            ReplAction::None
        }
    }

    fn delete(&mut self) -> ReplAction {
        if self.cursor < self.buffer.len() {
            // Remove char at cursor
            // remove() takes byte index and removes the char at that index.
            // We just need to make sure cursor is at char boundary, which we maintain.
            self.buffer.remove(self.cursor);
            ReplAction::Render
        } else {
            ReplAction::None
        }
    }

    fn move_left(&mut self) -> ReplAction {
        if self.cursor > 0 {
            if let Some(c) = self.buffer[..self.cursor].chars().next_back() {
                self.cursor -= c.len_utf8();
                ReplAction::Render
            } else {
                ReplAction::None
            }
        } else {
            ReplAction::None
        }
    }

    fn move_right(&mut self) -> ReplAction {
        if self.cursor < self.buffer.len() {
            if let Some(c) = self.buffer[self.cursor..].chars().next() {
                self.cursor += c.len_utf8();
                ReplAction::Render
            } else {
                ReplAction::None
            }
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
            if !full_code.trim().is_empty() && self.history.last() != Some(&full_code) {
                self.history.push(full_code.clone());
            }

            self.buffer.clear();
            self.cursor = 0;
            self.pending_block.clear();
            self.history_idx = None;
            self.is_multiline = false;

            // Expand macros before submitting
            let expanded_code = expand_macros(&full_code);

            ReplAction::Submit {
                code: expanded_code,
                last_line,
                prompt: current_prompt,
            }
        } else {
            self.pending_block = full_code;
            self.buffer.clear();
            self.cursor = 0;
            self.history_idx = None;
            self.is_multiline = true;

            ReplAction::AcceptLine {
                line: last_line,
                prompt: current_prompt,
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

        let tokens = Lexer::new(full_code.to_string()).scan_tokens();
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
                    }
                }
                _ => {}
            }
        }

        if balance > 0 || is_incomplete_string {
            return false;
        }

        // Special check for macro: if the line starts with ! and Lexer failed, we might want to submit
        // However, we rely on Lexer error to trigger expansion.
        // If `!ls` is entered, Lexer returns Error("Unexpected character !").
        // We catch that error in `should_execute`?
        // Currently `should_execute` returns false if balance > 0 or incomplete string.
        // If Lexer returns error "Unexpected character !", balance is 0, incomplete_string is false.
        // So `should_execute` proceeds to check line count / colon logic.
        // `!ls` -> 1 line, no colon -> returns true.
        // So it submits.
        // Then `handle_enter` calls `expand_macros`. `expand_macros` sees the error and expands.
        // Seems correct.

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

                    let mut new_lines: Vec<String> =
                        lines.iter().map(|s| s.to_string()).collect();
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
