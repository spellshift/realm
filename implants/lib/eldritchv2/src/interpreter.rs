use super::ast::{BuiltinFn, Environment, Expr, FStringSegment, Function, Stmt, Value};
use super::lexer::Lexer;
use super::parser::Parser;
use super::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// --- Helper Functions ---

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::None => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::String(s) => !s.is_empty(),
        Value::List(l) => !l.borrow().is_empty(),
        Value::Dictionary(d) => !d.borrow().is_empty(),
        Value::Function(_) | Value::NativeFunction(_, _) => true,
    }
}

// --- Interpreter ---

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    pub should_return: bool,
    pub return_value: Value,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment {
            parent: None,
            values: HashMap::new(),
        }));

        let mut interpreter = Interpreter {
            env,
            should_return: false,
            return_value: Value::None,
        };

        interpreter.define_builtins();
        interpreter
    }

    fn define_builtin(&mut self, name: &str, _arity: usize, func: BuiltinFn) {
        self.env.borrow_mut().values.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), func),
        );
    }

    fn define_builtins(&mut self) {
        self.define_builtin("print", 1, |args| {
            println!("{}", args[0].to_string());
            Ok(Value::None)
        });

        self.define_builtin("len", 1, |args| match &args[0] {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
            Value::Dictionary(d) => Ok(Value::Int(d.borrow().len() as i64)),
            _ => Err(format!("'len()' is not defined for type: {:?}", args[0])),
        });

        self.define_builtin("range", 2, |args| {
            let (start, end) = match args {
                [Value::Int(end)] => (0, *end),
                [Value::Int(start), Value::Int(end)] => (*start, *end),
                _ => return Err("Range expects one or two integer arguments.".to_string()),
            };

            let mut list = Vec::new();
            if start < end {
                for i in start..end {
                    list.push(Value::Int(i));
                }
            }
            Ok(Value::List(Rc::new(RefCell::new(list))))
        });

        self.define_builtin("input", 1, |_| {
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => Ok(Value::String(input.trim().to_string())),
                Err(e) => Err(format!("Input error: {}", e)),
            }
        });

        self.define_builtin("assert", 1, |args| {
            if !is_truthy(&args[0]) {
                return Err(format!(
                    "Assertion failed: value '{:?}' is not truthy",
                    args[0]
                ));
            }
            Ok(Value::None)
        });

        self.define_builtin("assert_eq", 2, |args| {
            if args[0] != args[1] {
                return Err(format!(
                    "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
                    args[0], args[1]
                ));
            }
            Ok(Value::None)
        });

        self.define_builtin("fail", 1, |args| {
            Err(format!("Test failed explicitly: {}", args[0].to_string()))
        });
    }

    pub fn execute_eval(&mut self, input: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        let eval_env = Rc::new(RefCell::new(Environment {
            parent: Some(Rc::clone(&self.env)),
            values: HashMap::new(),
        }));

        let mut eval_interp = Interpreter {
            env: eval_env,
            should_return: false,
            return_value: Value::None,
        };

        for stmt in ast {
            eval_interp.execute(&stmt)?;
        }

        Ok(eval_interp.return_value)
    }

    pub fn interpret(&mut self, input: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse()?;

        let mut last_val = Value::None;

        for stmt in stmts {
            match &stmt {
                Stmt::Expression(expr) => {
                    last_val = self.evaluate(expr)?;
                }
                _ => {
                    self.execute(&stmt)?;
                    if self.should_return {
                        let ret = self.return_value.clone();
                        self.should_return = false;
                        self.return_value = Value::None;
                        return Ok(ret);
                    }
                    last_val = Value::None;
                }
            }
        }
        Ok(last_val)
    }

    // --- Scope Helper ---

    // Updates an existing variable in the nearest scope, or defines it in the current scope if new.
    fn assign_variable(&mut self, name: &str, value: Value) {
        let mut env_opt = Some(Rc::clone(&self.env));
        let mut target_env = None;

        // Traverse up to find if the variable already exists
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

    // --- Execution Logic ---

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        if self.should_return {
            return Ok(());
        }

        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Stmt::Assignment(name, expr) => {
                let value = self.evaluate(expr)?;
                self.assign_variable(name, value);
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let eval_cond = &self.evaluate(condition)?;
                if is_truthy(eval_cond) {
                    // FIX: Execute in current scope, do not create new scope
                    self.execute_stmts(then_branch)?;
                } else if let Some(else_stmts) = else_branch {
                    // FIX: Execute in current scope
                    self.execute_stmts(else_stmts)?;
                }
            }
            Stmt::Return(expr) => {
                self.return_value = expr
                    .as_ref()
                    .map_or(Value::None, |e| self.evaluate(e).unwrap());
                self.should_return = true;
            }
            Stmt::Def(name, params, body) => {
                let func = Value::Function(Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.env.clone(),
                });
                self.env.borrow_mut().values.insert(name.clone(), func);
            }
            Stmt::For(ident, iterable, body) => {
                let iterable_val = self.evaluate(iterable)?;
                let list_rc = match iterable_val {
                    Value::List(rc) => rc,
                    _ => {
                        return Err(format!(
                            "'for' loop can only iterate over lists/iterables. Found {:?}",
                            iterable_val
                        ))
                    }
                };

                let list = list_rc.borrow();

                for item in list.iter() {
                    // FIX: Do not create a new environment for the loop.
                    // Assign loop variable in the current scope.
                    self.assign_variable(ident, item.clone());

                    self.execute_stmts(body)?;

                    if self.should_return {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    // New helper: executes a list of statements in the *current* environment
    fn execute_stmts(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            self.execute(stmt)?;
            if self.should_return {
                break;
            }
        }
        Ok(())
    }

    // --- Evaluation Logic ---

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Identifier(name) => self.lookup_variable(name),
            Expr::BinaryOp(left, op, right) => self.apply_binary_op(left, op, right),
            Expr::Call(callee, args) => self.call_function(callee, args),
            Expr::List(elements) => self.evaluate_list_literal(elements),
            Expr::Dictionary(entries) => self.evaluate_dict_literal(entries),
            Expr::Index(obj, index) => self.evaluate_index(obj, index),
            Expr::FString(segments) => self.evaluate_fstring(segments),
        }
    }

    fn evaluate_list_literal(&mut self, elements: &[Expr]) -> Result<Value, String> {
        let mut vals = Vec::new();
        for expr in elements {
            vals.push(self.evaluate(expr)?);
        }
        Ok(Value::List(Rc::new(RefCell::new(vals))))
    }

    fn evaluate_dict_literal(&mut self, entries: &[(Expr, Expr)]) -> Result<Value, String> {
        let mut map = HashMap::new();
        for (key_expr, value_expr) in entries {
            let key_val = self.evaluate(key_expr)?;
            let value_val = self.evaluate(value_expr)?;

            let key_str = match key_val {
                Value::String(s) => s,
                _ => return Err("Dictionary keys must be strings.".to_string()),
            };
            map.insert(key_str, value_val);
        }
        Ok(Value::Dictionary(Rc::new(RefCell::new(map))))
    }

    fn evaluate_index(&mut self, obj: &Expr, index: &Expr) -> Result<Value, String> {
        let obj_val = self.evaluate(obj)?;
        let idx_val = self.evaluate(index)?;

        match obj_val {
            Value::List(l) => {
                let idx_int = match idx_val {
                    Value::Int(i) => i,
                    _ => return Err("List indices must be integers".to_string()),
                };
                let list = l.borrow();
                if idx_int < 0 || idx_int as usize >= list.len() {
                    return Err("List index out of range".to_string());
                }
                Ok(list[idx_int as usize].clone())
            }
            Value::Dictionary(d) => {
                let key_str = match idx_val {
                    Value::String(s) => s,
                    _ => return Err("Dictionary keys must be strings".to_string()),
                };
                let dict = d.borrow();
                match dict.get(&key_str) {
                    Some(v) => Ok(v.clone()),
                    None => Err(format!("KeyError: '{}'", key_str)),
                }
            }
            _ => Err(format!("Type not subscriptable: {:?}", obj_val)),
        }
    }

    fn evaluate_fstring(&mut self, segments: &[FStringSegment]) -> Result<Value, String> {
        let mut parts = Vec::new();
        for segment in segments {
            match segment {
                FStringSegment::Literal(s) => parts.push(s.clone()),
                FStringSegment::Expression(expr) => {
                    let val = self.evaluate(expr)?;
                    parts.push(val.to_string());
                }
            }
        }
        Ok(Value::String(parts.join("")))
    }

    fn lookup_variable(&self, name: &str) -> Result<Value, String> {
        let mut current_env = Some(Rc::clone(&self.env));
        while let Some(env_rc) = current_env {
            let env_ref = env_rc.borrow();
            if let Some(value) = env_ref.values.get(name) {
                return Ok(value.clone());
            }
            current_env = env_ref.parent.clone();
        }
        Err(format!("Undefined variable: '{}'", name))
    }

    fn call_function(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, String> {
        let callee_val = self.evaluate(callee)?;
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }
        let args_slice = evaluated_args.as_slice();

        match callee_val {
            Value::NativeFunction(_, f) => f(args_slice),

            Value::Function(Function {
                name,
                params,
                body,
                closure,
            }) => {
                if params.len() != args_slice.len() {
                    return Err(format!(
                        "Function '{}' expected {} arguments but got {}.",
                        name,
                        params.len(),
                        args_slice.len()
                    ));
                }

                let function_env = Rc::new(RefCell::new(Environment {
                    parent: Some(closure),
                    values: HashMap::new(),
                }));

                for (param, arg) in params.iter().zip(args_slice.iter()) {
                    function_env
                        .borrow_mut()
                        .values
                        .insert(param.clone(), arg.clone());
                }

                let original_env = Rc::clone(&self.env);
                self.env = function_env;

                self.should_return = false;
                self.return_value = Value::None;

                let mut return_val = Value::None;
                for stmt in body {
                    self.execute(&stmt)?;
                    if self.should_return {
                        return_val = self.return_value.clone();
                        break;
                    }
                }

                self.env = original_env;

                self.should_return = false;
                self.return_value = Value::None;

                Ok(return_val)
            }
            _ => Err(format!("Cannot call value of type: {:?}", callee_val)),
        }
    }

    fn apply_binary_op(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Value, String> {
        let a = self.evaluate(left)?;
        let b = self.evaluate(right)?;

        match (a, op.clone(), b) {
            (a, Token::Eq, b) => Ok(Value::Bool(a == b)),
            (a, Token::NotEq, b) => Ok(Value::Bool(a != b)),
            (Value::Int(a), Token::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Token::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),

            (Value::Int(a), Token::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Int(a), Token::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Int(a), Token::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Int(a), Token::Slash, Value::Int(b)) => {
                if b == 0 {
                    return Err("attempt to divide by zero".to_string());
                }
                Ok(Value::Int(a / b))
            }
            (Value::String(a), Token::Plus, Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(format!(
                "Unsupported binary operation: {:?} {:?} {:?}",
                self.evaluate(left)?,
                op,
                self.evaluate(right)?
            )),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        is_truthy(value)
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::None => "None".to_string(),
            Value::Bool(b) => {
                if *b {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            Value::Int(i) => i.to_string(),
            Value::String(s) => s.clone(),
            Value::List(l) => format!(
                "[{}]",
                l.borrow()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Dictionary(d) => {
                let s: Vec<String> = d
                    .borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", s.join(", "))
            }
            Value::Function(f) => format!("<function {}>", f.name),
            Value::NativeFunction(name, _) => format!("<native function {}>", name),
        }
    }
}
