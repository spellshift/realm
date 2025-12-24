use super::super::ast::{BuiltinFn, Environment, Value};
use super::super::lexer::Lexer;
use super::super::parser::Parser;
use super::super::token::{Span, TokenKind};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

use super::builtins::{get_all_builtins, get_all_builtins_with_kwargs, get_stubs};
use super::error::{EldritchError, EldritchErrorKind, StackFrame};
use super::eval;
use super::exec;
use super::introspection::find_best_match;
use super::methods::get_native_methods;
use super::printer::{Printer, StdoutPrinter};
use crate::ast::ForeignValue;

#[derive(Clone, PartialEq)]
pub enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}

pub struct Interpreter {
    pub env: Arc<RwLock<Environment>>,
    pub flow: Flow,
    pub depth: usize,
    pub call_stack: Vec<StackFrame>,
    pub current_func_name: String,
    pub is_scope_owner: bool,
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        if self.is_scope_owner {
            // Break reference cycles by clearing the environment values.
            // This drops all variables including functions, which may hold references back to the environment.
            self.env.write().values.clear();
            self.env.write().parent = None;
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::new_with_printer(Arc::new(StdoutPrinter))
    }

    pub fn new_with_printer(printer: Arc<dyn Printer + Send + Sync>) -> Self {
        let env = Arc::new(RwLock::new(Environment {
            parent: None,
            values: BTreeMap::new(),
            printer,
            libraries: BTreeSet::new(),
        }));

        let mut interpreter = Interpreter {
            env,
            flow: Flow::Next,
            depth: 0,
            call_stack: Vec::new(),
            current_func_name: "<module>".to_string(),
            is_scope_owner: true,
        };

        interpreter.load_builtins();
        interpreter
    }

    fn load_builtins(&mut self) {
        for (name, func) in get_all_builtins() {
            self.register_function(name, func);
        }
        for (name, func) in get_all_builtins_with_kwargs() {
            self.env.write().values.insert(
                name.to_string(),
                Value::NativeFunctionWithKwargs(name.to_string(), func),
            );
        }
        for (name, func) in get_stubs() {
            self.register_function(name, func);
        }
        // Hardcoded pass variable for now
        self.env
            .write()
            .values
            .insert("pass".to_string(), Value::None);
    }

    pub fn register_function(&mut self, name: &str, func: BuiltinFn) {
        self.env.write().values.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), func),
        );
    }

    pub fn register_module(&mut self, name: &str, module: Value) {
        // Ensure the value is actually a dictionary or structurally appropriate for a module
        // We accept any Value, but practically it should be a Dictionary of functions
        self.env.write().values.insert(name.to_string(), module);
    }

    pub fn register_lib(&mut self, val: impl ForeignValue + 'static) {
        let name = val.type_name().to_string();
        self.env.write().libraries.insert(name.clone());
        self.env
            .write()
            .values
            .insert(name, Value::Foreign(Arc::new(val)));
    }

    // Helper to create errors from interpreter context
    pub fn error<T>(
        &self,
        kind: EldritchErrorKind,
        msg: &str,
        span: Span,
    ) -> Result<T, EldritchError> {
        // Construct the error with the current call stack
        let mut err = EldritchError::new(kind, msg, span);
        // We attach the full stack of callers.
        // We also want to include the current frame's context as the final location if not already in stack?
        // No, the traceback convention is:
        // Stack Frame 0 (bottom): <module> line X
        // ...
        // Current: <func> line Y (error location)

        let mut full_stack = self.call_stack.clone();
        full_stack.push(StackFrame {
            name: self.current_func_name.clone(),
            filename: "<script>".to_string(),
            line: span.line,
        });

        err = err.with_stack(full_stack);
        Err(err)
    }

    // Legacy support wrapper
    pub fn runtime_error<T>(&self, msg: &str, span: Span) -> Result<T, EldritchError> {
        self.error(EldritchErrorKind::RuntimeError, msg, span)
    }

    pub fn push_frame(&mut self, name: &str, span: Span) {
        // Push the CURRENT context onto the stack before entering new function.
        // `name` is the name of the function being CALLED (new context).
        // `span` is the call site in the CURRENT context.

        self.call_stack.push(StackFrame {
            name: self.current_func_name.clone(),
            filename: "<script>".to_string(),
            line: span.line,
        });
        self.current_func_name = name.to_string();
    }

    pub fn pop_frame(&mut self) {
        if let Some(frame) = self.call_stack.pop() {
            self.current_func_name = frame.name;
        } else {
            // Should not happen if push/pop balanced correctly, but fallback safe
            self.current_func_name = "<module>".to_string();
        }
    }

    pub fn interpret(&mut self, input: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = match lexer.scan_tokens() {
            Ok(t) => t,
            Err(e) => return Err(format!("Lexer Error: {e}")),
        };
        let mut parser = Parser::new(tokens);
        let stmts = match parser.parse() {
            Ok(s) => s,
            Err(e) => return Err(format!("Parser Error: {e}")),
        };

        let mut last_val = Value::None;

        // Reset state for fresh run
        self.call_stack.clear();
        self.current_func_name = "<module>".to_string();

        for stmt in stmts {
            match &stmt.kind {
                // Special case: if top-level statement is an expression, return its value
                // This matches behavior of typical REPLs / starlark-like exec
                super::super::ast::StmtKind::Expression(expr) => {
                    let res = eval::evaluate(self, expr);
                    match res {
                        Ok(v) => last_val = v,
                        Err(e) => {
                            return Err(self.format_error(input, e));
                        }
                    }
                }
                _ => {
                    let res = exec::execute(self, &stmt);
                    match res {
                        Ok(_) => {
                            if let Flow::Return(v) = &self.flow {
                                let ret = v.clone();
                                self.flow = Flow::Next;
                                return Ok(ret);
                            }
                            last_val = Value::None;
                        }
                        Err(e) => {
                            return Err(self.format_error(input, e));
                        }
                    }
                }
            }
        }
        Ok(last_val)
    }

    pub(crate) fn format_error(&self, source: &str, error: EldritchError) -> String {
        // combine formatted traceback with source snippet
        let mut output = error.to_string();

        let lines: Vec<&str> = source.lines().collect();
        if error.span.line > 0 && error.span.line <= lines.len() {
            let line_idx = error.span.line - 1;
            let line_content = lines[line_idx];

            // Calculate column offset relative to trimmed line
            let leading_whitespace = line_content.len() - line_content.trim_start().len();

            // Calculate line start byte offset
            let mut line_start = error.span.start;
            let source_bytes = source.as_bytes();
            // Walk backwards from error start to find newline
            // If error.span.start is beyond source len (shouldn't happen for valid error), clamp it.
            if line_start > source_bytes.len() {
                line_start = source_bytes.len();
            }
            while line_start > 0 && source_bytes[line_start - 1] != b'\n' {
                line_start -= 1;
            }

            // Calculate raw column (byte offset from start of line)
            let raw_col = error.span.start.saturating_sub(line_start);

            // Calculate display column relative to trimmed string
            let display_col = raw_col.saturating_sub(leading_whitespace);

            // Create dynamic padding
            let padding = format!("{:>width$}", "", width = display_col);

            output.push_str(&format!(
                "\n\nError location:\n  at line {}:\n    {}\n    {}^-- here",
                error.span.line,
                line_content.trim(),
                padding
            ));
        }
        output
    }

    pub(crate) fn assign_variable(&mut self, name: &str, value: Value) {
        let mut env_opt = Some(self.env.clone());
        let mut target_env = None;
        while let Some(env) = env_opt {
            if env.read().values.contains_key(name) {
                target_env = Some(env.clone());
                break;
            }
            env_opt = env.read().parent.clone();
        }
        if let Some(env) = target_env {
            env.write().values.insert(name.to_string(), value);
        } else {
            self.env.write().values.insert(name.to_string(), value);
        }
    }

    pub fn define_variable(&mut self, name: &str, value: Value) {
        self.env.write().values.insert(name.to_string(), value);
    }

    pub fn lookup_variable(&self, name: &str, span: Span) -> Result<Value, EldritchError> {
        let mut current_env = Some(self.env.clone());
        while let Some(env_arc) = current_env {
            let env_ref = env_arc.read();
            if let Some(value) = env_ref.values.get(name) {
                return Ok(value.clone());
            }
            current_env = env_ref.parent.clone();
        }

        // If variable is not found, try to find a suggestion
        let mut candidates = Vec::new();
        let mut current_env = Some(self.env.clone());
        while let Some(env_arc) = current_env {
            let env_ref = env_arc.read();
            for k in env_ref.values.keys() {
                candidates.push(k.clone());
            }
            current_env = env_ref.parent.clone();
        }

        let mut msg = format!("Undefined variable: '{name}'");
        if let Some(suggestion) = find_best_match(name, &candidates) {
            msg.push_str(&format!("\nDid you mean '{suggestion}'?"));
        }

        self.error(EldritchErrorKind::NameError, &msg, span)
    }

    pub fn complete(&self, code: &str, cursor: usize) -> (usize, Vec<String>) {
        let mut candidates = BTreeSet::new();
        let mut prefix = String::new();

        // 1. Tokenize the input string up to cursor to understand context
        // If the code has syntax errors (which is likely during typing), the lexer might fail.
        // We'll try to extract the relevant part near the cursor.
        // Simple approach: look at the line up to cursor.
        let safe_cursor = if cursor <= code.len() {
            // Ensure cursor is at a char boundary. If not, floor it.
            let mut c = cursor;
            while c > 0 && !code.is_char_boundary(c) {
                c -= 1;
            }
            c
        } else {
            code.len()
        };

        let line_up_to_cursor = &code[..safe_cursor];

        // If empty, return nothing
        if line_up_to_cursor.trim().is_empty() {
            return (cursor, Vec::new());
        }

        // Try to find the token we are on.
        // We can scan the whole string, but we really care about the last token(s).
        let mut lexer = Lexer::new(line_up_to_cursor.to_string());
        let tokens = match lexer.scan_tokens() {
            Ok(t) => t,
            // If scanning fails (e.g. open string), we might still want to try?
            // For now, if lexer fails, we fallback to simple word splitting or empty.
            Err(_) => {
                // Fallback: Check if we are typing a simple identifier at end
                let last_word = line_up_to_cursor
                    .split_terminator(|c: char| !c.is_alphanumeric() && c != '_')
                    .next_back()
                    .unwrap_or("");
                if !last_word.is_empty() {
                    prefix = last_word.to_string();
                }
                // If we can't tokenize, we assume we can't do dot-access analysis safely.
                // But we can still try to suggest globals matching the prefix.
                // We return empty tokens list, but allow execution to proceed to prefix filtering.
                Vec::new()
            }
        };

        // Determine context from tokens
        let mut target_val: Option<Value> = None;
        let meaningful_tokens: Vec<&super::super::token::Token> = tokens
            .iter()
            .filter(|t| {
                t.kind != TokenKind::Eof
                    && t.kind != TokenKind::Newline
                    && t.kind != TokenKind::Indent
                    && t.kind != TokenKind::Dedent
            })
            .collect();

        if !meaningful_tokens.is_empty() {
            let last_token = meaningful_tokens[meaningful_tokens.len() - 1];

            // Only consider the token if the cursor is at the end of it, or if it's a Dot
            // If there is whitespace after the token (cursor > end), we treat it as empty prefix
            // unless it is a Dot which implies property access continuation.
            let is_touching = last_token.span.end == cursor;

            if is_touching || last_token.kind == TokenKind::Dot {
                // Case 1: Cursor is after a Dot (object property access)
                // e.g. "foo."
                if last_token.kind == TokenKind::Dot {
                    // Look at the token before dot
                    if meaningful_tokens.len() >= 2 {
                        let obj_token = meaningful_tokens[meaningful_tokens.len() - 2];
                        match &obj_token.kind {
                            TokenKind::Identifier(name) => {
                                // Resolve variable
                                if let Ok(val) = self.lookup_variable(name, Span::new(0, 0, 0)) {
                                    target_val = Some(val);
                                }
                            }
                            TokenKind::String(s) => {
                                target_val = Some(Value::String(s.clone()));
                            }
                            TokenKind::RBracket => {
                                // Assume List
                                target_val = Some(Value::List(Arc::new(RwLock::new(Vec::new()))));
                            }
                            TokenKind::RBrace => {
                                // Assume Dictionary (could be Set, but Dict is safer default)
                                target_val = Some(Value::Dictionary(Arc::new(RwLock::new(
                                    BTreeMap::new(),
                                ))));
                            }
                            TokenKind::RParen => {
                                // Assume Tuple
                                target_val = Some(Value::Tuple(Vec::new()));
                            }
                            _ => {}
                        }
                    }
                }
                // Case 2: Cursor is at an Identifier (completing current word)
                // e.g. "fo" or "foo.b"
                else if let TokenKind::Identifier(name) = &last_token.kind {
                    #[allow(clippy::collapsible_if)]
                    if is_touching {
                        prefix = name.clone();

                        // Check if the previous token was a Dot
                        if meaningful_tokens.len() >= 2
                            && meaningful_tokens[meaningful_tokens.len() - 2].kind == TokenKind::Dot
                        {
                            // Object property completion
                            if meaningful_tokens.len() >= 3 {
                                let obj_token = meaningful_tokens[meaningful_tokens.len() - 3];
                                match &obj_token.kind {
                                    TokenKind::Identifier(obj_name) => {
                                        if let Ok(val) =
                                            self.lookup_variable(obj_name, Span::new(0, 0, 0))
                                        {
                                            target_val = Some(val);
                                        }
                                    }
                                    TokenKind::String(s) => {
                                        target_val = Some(Value::String(s.clone()));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        } else if !prefix.is_empty() {
            // Lexer failed but we extracted a prefix manually
        }

        if let Some(val) = target_val {
            // Suggest methods/properties of val
            match &val {
                Value::Foreign(obj) => {
                    for m in obj.method_names() {
                        candidates.insert(m);
                    }
                }
                Value::Dictionary(_) => {
                    // Keys as suggestions?
                    // Usually dict keys are accessed via [], not dot.
                    // But maybe we want to suggest methods like .keys(), .items()?
                    // Python dicts have methods.
                    for m in get_native_methods(&val) {
                        candidates.insert(m);
                    }
                    // If the user wants keys, they'd type d["..."]
                }
                _ => {
                    for m in get_native_methods(&val) {
                        candidates.insert(m);
                    }
                }
            }
        } else {
            // Global completion (variables, builtins, keywords)
            // Only do this if we are NOT in a property access context (target_val is None)
            // And also check we didn't fail to find target_val when we should have (e.g. undefined var).
            // If tokens said "obj.prop", and we couldn't resolve obj, target_val is None.
            // In that case, we shouldn't suggest globals.

            // We only suggest globals if we couldn't resolve a target value AND
            // we are not in a dot context that failed to resolve.
            // If tokens said "obj.prop" but target_val is None, it means "obj" is invalid/undefined.
            // In that case, we should probably return empty, OR if prefix matches globals, return globals?
            // "abc.d" -> if abc is undefined, we shouldn't suggest globals starting with d.

            let mut is_dot_access = false;
            if !meaningful_tokens.is_empty() {
                let last = meaningful_tokens[meaningful_tokens.len() - 1];
                if last.kind == TokenKind::Dot
                    || (matches!(last.kind, TokenKind::Identifier(_))
                        && meaningful_tokens.len() >= 2
                        && meaningful_tokens[meaningful_tokens.len() - 2].kind == TokenKind::Dot)
                {
                    is_dot_access = true;
                }
            }

            if !is_dot_access {
                // 1. Keywords
                let keywords = vec![
                    "def", "if", "elif", "else", "return", "for", "in", "True", "False", "None",
                    "and", "or", "not", "break", "continue", "pass", "lambda",
                ];
                for kw in keywords {
                    candidates.insert(kw.to_string());
                }

                // 2. Builtins & Variables (walk up the environment chain)
                let mut current_env = Some(self.env.clone());
                while let Some(env_arc) = current_env {
                    let env_ref = env_arc.read();
                    for key in env_ref.values.keys() {
                        candidates.insert(key.clone());
                    }
                    current_env = env_ref.parent.clone();
                }
            }
        }

        // Filter by prefix
        let results: Vec<String> = candidates
            .into_iter()
            .filter(|c| {
                c.starts_with(&prefix)
                    && *c != prefix
                    && (!c.starts_with('_') || prefix.starts_with('_'))
            })
            .collect();

        // Calculate start index for completion replacement
        // Typically cursor - prefix.len()
        let start_index = if cursor >= prefix.len() {
            cursor - prefix.len()
        } else {
            0
        };

        (start_index, results)
    }
}
