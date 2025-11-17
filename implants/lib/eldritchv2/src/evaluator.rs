use crate::ast::{BlockStatement, Expression, Program, Statement};
use crate::lexer::Token;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

/// Runtime values produced by evaluating the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),

    /// User-defined functions in the DSL.
    Function(FunctionValue),

    /// Native Rust function exposed to the DSL.
    ///
    /// Signature: fn(&[Value]) -> Result<Value, EvalError>
    NativeFunction(NativeFn),

    /// Used internally to implement `return` in functions.
    Return(Box<Value>),

    /// Equivalent of `None` / `null` / `NoneType`.
    Null,
}

/// Function object for DSL-defined functions.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub parameters: Vec<String>,
    pub body: BlockStatement,
    pub env: EnvRef,
}

/// Native function type: simple function pointer.
///
/// You can adapt any `fn(i64, i64) -> i64` or similar into this.
pub type NativeFn = fn(&[Value]) -> Result<Value, EvalError>;

/// Simple evaluation error type.
#[derive(Debug, Clone)]
pub struct EvalError {
    message: String,
}

impl EvalError {
    pub fn new(message: String) -> Self {
        EvalError { message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Shared, mutable environment reference.
pub type EnvRef = Rc<RefCell<Environment>>;

/// Variable environment, with optional outer (for lexical scoping).
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: BTreeMap<String, Value>,
    outer: Option<EnvRef>,
}

impl Environment {
    /// Create a new, empty environment.
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            store: BTreeMap::new(),
            outer: None,
        }))
    }

    /// Create a new environment that chains to an outer one (used for functions).
    pub fn new_enclosed(outer: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            store: BTreeMap::new(),
            outer: Some(outer),
        }))
    }

    /// Define or update a variable.
    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    /// Get a variable by name, searching outer scopes if necessary.
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.store.get(name) {
            Some(v.clone())
        } else if let Some(outer) = &self.outer {
            outer.borrow().get(name)
        } else {
            None
        }
    }

    /// Convenience method: register a native function.
    pub fn set_native_fn(&mut self, name: &str, f: NativeFn) {
        self.set(name.to_string(), Value::NativeFunction(f));
    }

    /// Build an environment from a slice of (name, native_fn) pairs.
    pub fn with_builtins(builtins: &[(&str, NativeFn)]) -> EnvRef {
        let env = Environment::new();
        {
            let mut e = env.borrow_mut();
            for (name, f) in builtins {
                e.set((*name).to_string(), Value::NativeFunction(*f));
            }
        }
        env
    }
}

/// The evaluator walks the AST and produces runtime values.
pub struct Evaluator {
    env: EnvRef,
}

impl Evaluator {
    /// Create an evaluator with a fresh, empty environment.
    pub fn new() -> Self {
        Evaluator {
            env: Environment::new(),
        }
    }

    /// Create an evaluator using an existing environment (with builtins, etc.).
    pub fn with_env(env: EnvRef) -> Self {
        Evaluator { env }
    }

    /// Access the underlying environment (e.g. to inspect variables after running).
    pub fn env(&self) -> EnvRef {
        Rc::clone(&self.env)
    }

    /// Evaluate a whole program.
    pub fn eval_program(&mut self, program: &Program) -> Result<Value, EvalError> {
        let mut result = Value::Null;

        for stmt in &program.statements {
            result = self.eval_statement(stmt)?;

            // Propagate return up to the program level.
            if let Value::Return(inner) = result {
                return Ok(*inner);
            }
        }

        Ok(result)
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<Value, EvalError> {
        match stmt {
            Statement::Assign(name, expr) => {
                let val = self.eval_expression(expr)?;
                self.env.borrow_mut().set(name.clone(), val.clone());
                Ok(val)
            }
            Statement::Return(expr) => {
                let val = self.eval_expression(expr)?;
                Ok(Value::Return(Box::new(val)))
            }
            Statement::Expression(expr) => self.eval_expression(expr),
        }
    }

    fn eval_block_statement(&mut self, block: &BlockStatement) -> Result<Value, EvalError> {
        let mut result = Value::Null;

        for stmt in &block.statements {
            result = self.eval_statement(stmt)?;

            // Early-return from blocks if we see a `Return`.
            if let Value::Return(_) = result {
                return Ok(result);
            }
        }

        Ok(result)
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, EvalError> {
        match expr {
            Expression::IntegerLiteral(i) => Ok(Value::Integer(*i)),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Identifier(name) => self
                .env
                .borrow()
                .get(name)
                .ok_or_else(|| EvalError::new(format!("unknown identifier: {}", name))),

            Expression::Prefix(op, right) => {
                let right_val = self.eval_expression(right)?;
                self.eval_prefix_expression(op, right_val)
            }

            Expression::Infix(op, left, right) => {
                let left_val = self.eval_expression(left)?;
                let right_val = self.eval_expression(right)?;
                self.eval_infix_expression(op, left_val, right_val)
            }

            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond_val = self.eval_expression(condition)?;
                if self.is_truthy(&cond_val) {
                    self.eval_block_statement(consequence)
                } else if let Some(alt_block) = alternative {
                    self.eval_block_statement(alt_block)
                } else {
                    Ok(Value::Null)
                }
            }

            Expression::FunctionLiteral { parameters, body } => {
                let func = FunctionValue {
                    parameters: parameters.clone(),
                    body: body.clone(),
                    env: Rc::clone(&self.env),
                };
                Ok(Value::Function(func))
            }

            Expression::Call {
                function,
                arguments,
            } => {
                let func_val = self.eval_expression(function)?;

                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.eval_expression(arg)?);
                }

                self.apply_function(func_val, arg_values)
            }

            Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
            // If you later add more expression variants, handle them here.
        }
    }

    fn eval_prefix_expression(&self, op: &Token, right: Value) -> Result<Value, EvalError> {
        match op {
            Token::Bang => Ok(Value::Boolean(!self.is_truthy(&right))),
            Token::Minus => match right {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                other => Err(EvalError::new(format!("unknown operator: -{:?}", other))),
            },
            _ => Err(EvalError::new(format!("unknown prefix operator: {:?}", op))),
        }
    }

    fn eval_infix_expression(
        &self,
        op: &Token,
        left: Value,
        right: Value,
    ) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => self.eval_integer_infix(op, l, r),
            (Value::Boolean(l), Value::Boolean(r)) => self.eval_boolean_infix(op, l, r),
            (l, r) => match op {
                Token::Equal => Ok(Value::Boolean(l == r)),
                Token::NotEqual => Ok(Value::Boolean(l != r)),
                _ => Err(EvalError::new(format!(
                    "type mismatch in infix: {:?} {:?} {:?}",
                    l, op, r
                ))),
            },
        }
    }

    fn eval_integer_infix(&self, op: &Token, left: i64, right: i64) -> Result<Value, EvalError> {
        match op {
            Token::Plus => Ok(Value::Integer(left + right)),
            Token::Minus => Ok(Value::Integer(left - right)),
            Token::Asterisk => Ok(Value::Integer(left * right)),
            Token::Slash => Ok(Value::Integer(left / right)), // no divide-by-zero guard yet

            Token::LessThan => Ok(Value::Boolean(left < right)),
            Token::GreaterThan => Ok(Value::Boolean(left > right)),
            Token::Equal => Ok(Value::Boolean(left == right)),
            Token::NotEqual => Ok(Value::Boolean(left != right)),

            _ => Err(EvalError::new(format!(
                "unknown operator for integers: {:?}",
                op
            ))),
        }
    }

    fn eval_boolean_infix(&self, op: &Token, left: bool, right: bool) -> Result<Value, EvalError> {
        match op {
            Token::Equal => Ok(Value::Boolean(left == right)),
            Token::NotEqual => Ok(Value::Boolean(left != right)),
            _ => Err(EvalError::new(format!(
                "unknown operator for booleans: {:?}",
                op
            ))),
        }
    }

    fn is_truthy(&self, v: &Value) -> bool {
        match v {
            Value::Boolean(b) => *b,
            Value::Null => false,
            // Simple rule: everything else is truthy.
            _ => true,
        }
    }

    fn apply_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
        match func {
            Value::Function(f) => {
                // Create a new environment chained to the function's defining environment.
                let extended_env = Environment::new_enclosed(Rc::clone(&f.env));

                {
                    let mut env_mut = extended_env.borrow_mut();
                    for (param, arg) in f.parameters.iter().zip(args.into_iter()) {
                        env_mut.set(param.clone(), arg);
                    }
                }

                // Evaluate body in the extended environment.
                let mut inner_eval = Evaluator::with_env(extended_env);
                let result = inner_eval.eval_block_statement(&f.body)?;

                // Unwrap return if needed.
                if let Value::Return(inner) = result {
                    Ok(*inner)
                } else {
                    Ok(result)
                }
            }
            Value::NativeFunction(fptr) => fptr(&args),
            other => Err(EvalError::new(format!(
                "attempted to call non-function value: {:?}",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    pub fn check_parser_errors(parser: &Parser) {
        if parser.errors().is_empty() {
            return;
        }
        for err in parser.errors() {
            eprintln!("parser error: {}", err);
        }
        panic!("parser had {} error(s)", parser.errors().len());
    }

    #[test]
    fn test_string_literal_evaluation() {
        let input = r#"
            let s = "hello";
            s;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        let env = Environment::new();
        let mut evaluator = Evaluator::with_env(env);

        let result = evaluator
            .eval_program(&program)
            .expect("evaluation should succeed");

        assert_eq!(
            result,
            Value::String("hello".to_string()),
            "expected string 'hello', got {:?}",
            result
        );
    }
}
