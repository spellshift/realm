use super::super::ast::{BuiltinFn, Environment, Value};
use super::super::lexer::Lexer;
use super::super::parser::Parser;
use super::super::token::Span;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

use super::builtins::{get_all_builtins, get_all_builtins_with_kwargs, get_stubs};
use super::error::{runtime_error, EldritchError};
use super::eval;
use super::exec;
use crate::lang::global_libs::get_global_libraries;

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

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment {
            parent: None,
            values: BTreeMap::new(),
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
            Err(e) => return Err(format!("Lexer Error: {}", e)),
        };
        let mut parser = Parser::new(tokens);
        let stmts = match parser.parse() {
            Ok(s) => s,
            Err(e) => return Err(format!("Parser Error: {}", e)),
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
        runtime_error(span, &format!("Undefined variable: '{}'", name))
    }
}
