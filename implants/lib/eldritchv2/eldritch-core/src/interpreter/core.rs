use super::super::ast::{BuiltinFn, Environment, Value};
use super::super::lexer::Lexer;
use super::super::parser::Parser;
use super::super::token::{Span, TokenKind};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use eldritch_hostcontext::*;

use super::builtins::{get_all_builtins, get_all_builtins_with_kwargs, get_stubs};
use super::error::{runtime_error, EldritchError};
use super::eval;
use super::exec;
use super::methods::get_native_methods;
use super::printer::{Printer, StdoutPrinter};
use crate::global_libs::get_global_libraries;

#[derive(Clone, PartialEq)]
pub enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    pub flow: Flow,
    pub depth: usize,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Dummy Host Implementation for Default
#[derive(Debug)]
struct DefaultHost;
impl HostContext for DefaultHost {
    fn list_dir(&self, _req: ListDirRequest) -> Result<ListDirResponse, String> { Err("Not implemented".to_string()) }
    fn file_read(&self, _req: FileReadRequest) -> Result<FileReadResponse, String> { Err("Not implemented".to_string()) }
    fn file_write(&self, _req: FileWriteRequest) -> Result<FileWriteResponse, String> { Err("Not implemented".to_string()) }
    fn file_remove(&self, _req: FileRemoveRequest) -> Result<FileRemoveResponse, String> { Err("Not implemented".to_string()) }
    fn process_list(&self, _req: ProcessListRequest) -> Result<ProcessListResponse, String> { Err("Not implemented".to_string()) }
    fn process_kill(&self, _req: ProcessKillRequest) -> Result<ProcessKillResponse, String> { Err("Not implemented".to_string()) }
    fn exec(&self, _req: ExecRequest) -> Result<ExecResponse, String> { Err("Not implemented".to_string()) }
    fn sys_info(&self, _req: SysInfoRequest) -> Result<SysInfoResponse, String> { Err("Not implemented".to_string()) }
    fn env_get(&self, _req: EnvGetRequest) -> Result<EnvGetResponse, String> { Err("Not implemented".to_string()) }
    fn env_set(&self, _req: EnvSetRequest) -> Result<EnvSetResponse, String> { Err("Not implemented".to_string()) }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::new_with_printer(Arc::new(StdoutPrinter))
    }

    pub fn new_with_printer(printer: Arc<dyn Printer + Send + Sync>) -> Self {
        Self::new_with_host(printer, Arc::new(DefaultHost))
    }

    pub fn new_with_host(printer: Arc<dyn Printer + Send + Sync>, host: Arc<dyn HostContext>) -> Self {
        let env = Rc::new(RefCell::new(Environment {
            parent: None,
            values: BTreeMap::new(),
            printer,
            host,
        }));

        let mut interpreter = Interpreter {
            env,
            flow: Flow::Next,
            depth: 0,
        };

        interpreter.load_builtins();
        interpreter.load_libraries();
        interpreter
    }

    fn load_builtins(&mut self) {
        for (name, func) in get_all_builtins() {
            self.register_function(name, func);
        }
        for (name, func) in get_all_builtins_with_kwargs() {
            self.env.borrow_mut().values.insert(
                name.to_string(),
                Value::NativeFunctionWithKwargs(name.to_string(), func),
            );
        }
        for (name, func) in get_stubs() {
            self.register_function(name, func);
        }
        // Hardcoded pass variable for now
        self.env
            .borrow_mut()
            .values
            .insert("pass".to_string(), Value::None);
    }

    fn load_libraries(&mut self) {
        let libs = get_global_libraries();
        for (name, val) in libs {
            self.env
                .borrow_mut()
                .values
                .insert(name, Value::Foreign(val));
        }
    }

    pub fn register_function(&mut self, name: &str, func: BuiltinFn) {
        self.env.borrow_mut().values.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), func),
        );
    }

    pub fn register_module(&mut self, name: &str, module: Value) {
        // Ensure the value is actually a dictionary or structurally appropriate for a module
        // We accept any Value, but practically it should be a Dictionary of functions
        self.env
            .borrow_mut()
            .values
            .insert(name.to_string(), module);
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

        for stmt in stmts {
            match &stmt.kind {
                // Special case: if top-level statement is an expression, return its value
                // This matches behavior of typical REPLs / starlark-like exec
                super::super::ast::StmtKind::Expression(expr) => {
                    last_val =
                        eval::evaluate(self, expr).map_err(|e| self.format_error(input, e))?;
                }
                _ => {
                    exec::execute(self, &stmt).map_err(|e| self.format_error(input, e))?;
                    if let Flow::Return(v) = &self.flow {
                        let ret = v.clone();
                        self.flow = Flow::Next;
                        return Ok(ret);
                    }
                    last_val = Value::None;
                }
            }
        }
        Ok(last_val)
    }

    pub(crate) fn format_error(&self, source: &str, error: EldritchError) -> String {
        let lines: Vec<&str> = source.lines().collect();
        if error.span.line > 0 && error.span.line <= lines.len() {
            let line_idx = error.span.line - 1;
            let line_content = lines[line_idx];
            format!(
                "{}\n  at line {}:\n    {}\n    ^-- here",
                error.message,
                error.span.line,
                line_content.trim()
            )
        } else {
            format!("Error: {}", error.message)
        }
    }

    pub(crate) fn assign_variable(&mut self, name: &str, value: Value) {
        let mut env_opt = Some(Rc::clone(&self.env));
        let mut target_env = None;
        while let Some(env) = env_opt {
            if env.borrow().values.contains_key(name) {
                target_env = Some(env.clone());
                break;
            }
            env_opt = env.borrow().parent.clone();
        }
        if let Some(env) = target_env {
            env.borrow_mut().values.insert(name.to_string(), value);
        } else {
            self.env.borrow_mut().values.insert(name.to_string(), value);
        }
    }

    pub(crate) fn define_variable(&mut self, name: &str, value: Value) {
        self.env.borrow_mut().values.insert(name.to_string(), value);
    }

    pub(crate) fn lookup_variable(&self, name: &str, span: Span) -> Result<Value, EldritchError> {
        let mut current_env = Some(Rc::clone(&self.env));
        while let Some(env_rc) = current_env {
            let env_ref = env_rc.borrow();
            if let Some(value) = env_ref.values.get(name) {
                return Ok(value.clone());
            }
            current_env = env_ref.parent.clone();
        }
        runtime_error(span, &format!("Undefined variable: '{name}'"))
    }

    pub fn complete(&self, code: &str, cursor: usize) -> Vec<String> {
        let mut candidates = BTreeSet::new();
        let mut prefix = String::new();

        // 1. Tokenize the input string up to cursor to understand context
        // If the code has syntax errors (which is likely during typing), the lexer might fail.
        // We'll try to extract the relevant part near the cursor.
        // Simple approach: look at the line up to cursor.
        let line_up_to_cursor = if cursor <= code.len() {
            &code[..cursor]
        } else {
            code
        };

        // If empty, return nothing
        if line_up_to_cursor.trim().is_empty() {
            return Vec::new();
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
                    .last()
                    .unwrap_or("");
                if !last_word.is_empty() {
                    prefix = last_word.to_string();
                } else {
                    return Vec::new();
                }
                Vec::new() // Actually let's just use the prefix matching below if tokens fail
            }
        };

        // Determine context from tokens
        let mut target_val: Option<Value> = None;

        if !tokens.is_empty() {
            let last_token = &tokens[tokens.len() - 1];

            // Case 1: Cursor is after a Dot (object property access)
            // e.g. "foo."
            if last_token.kind == TokenKind::Dot {
                // Look at the token before dot
                if tokens.len() >= 2 {
                    let obj_token = &tokens[tokens.len() - 2];
                    match &obj_token.kind {
                        TokenKind::Identifier(name) => {
                            // Resolve variable
                            if let Ok(val) = self.lookup_variable(
                                name,
                                Span::new(0, 0, 0), // Dummy span
                            ) {
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
            // Case 2: Cursor is at an Identifier (completing current word)
            // e.g. "fo" or "foo.b"
            else if let TokenKind::Identifier(name) = &last_token.kind {
                prefix = name.clone();

                // Check if the previous token was a Dot
                if tokens.len() >= 2 && tokens[tokens.len() - 2].kind == TokenKind::Dot {
                    // Object property completion
                    if tokens.len() >= 3 {
                        let obj_token = &tokens[tokens.len() - 3];
                        match &obj_token.kind {
                            TokenKind::Identifier(obj_name) => {
                                if let Ok(val) = self.lookup_variable(
                                    obj_name,
                                    Span::new(0, 0, 0),
                                ) {
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
            let mut is_dot_access = false;
            if !tokens.is_empty() {
                let last = &tokens[tokens.len() - 1];
                if last.kind == TokenKind::Dot {
                    is_dot_access = true;
                } else if let TokenKind::Identifier(_) = last.kind {
                    if tokens.len() >= 2 && tokens[tokens.len() - 2].kind == TokenKind::Dot {
                        is_dot_access = true;
                    }
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
                let mut current_env = Some(Rc::clone(&self.env));
                while let Some(env_rc) = current_env {
                    let env_ref = env_rc.borrow();
                    for key in env_ref.values.keys() {
                        candidates.insert(key.clone());
                    }
                    current_env = env_ref.parent.clone();
                }
            }
        }

        // Filter by prefix
        candidates
            .into_iter()
            .filter(|c| c.starts_with(&prefix) && *c != prefix)
            .collect()
    }
}
