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
        Value::Tuple(t) => !t.is_empty(),
        Value::Function(_) | Value::NativeFunction(_, _) | Value::BoundMethod(_, _) => true,
    }
}

fn get_type_name(value: &Value) -> String {
    match value {
        Value::None => "NoneType".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Int(_) => "int".to_string(),
        Value::String(_) => "string".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Dictionary(_) => "dict".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Function(_) | Value::NativeFunction(_, _) | Value::BoundMethod(_, _) => {
            "function".to_string()
        }
    }
}

fn normalize_index(idx: i64, len: usize) -> usize {
    let len_i64 = len as i64;
    if idx < 0 {
        if idx + len_i64 < 0 {
            0
        } else {
            (idx + len_i64) as usize
        }
    } else {
        if idx > len_i64 {
            len
        } else {
            idx as usize
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}

// --- Interpreter ---

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    pub flow: Flow,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment {
            parent: None,
            values: HashMap::new(),
        }));

        let mut interpreter = Interpreter {
            env,
            flow: Flow::Next,
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
            Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
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

        self.define_builtin("type", 1, |args| Ok(Value::String(get_type_name(&args[0]))));
        self.define_builtin("bool", 1, |args| Ok(Value::Bool(is_truthy(&args[0]))));
        self.define_builtin("str", 1, |args| Ok(Value::String(args[0].to_string())));
        self.define_builtin("int", 1, |args| match &args[0] {
            Value::Int(i) => Ok(Value::Int(*i)),
            Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
            Value::String(s) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| format!("invalid literal for int(): '{}'", s)),
            _ => Err(format!(
                "int() argument must be a string, bytes or number, not '{}'",
                get_type_name(&args[0])
            )),
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
            flow: Flow::Next,
        };

        for stmt in ast {
            eval_interp.execute(&stmt)?;
        }

        if let Flow::Return(v) = eval_interp.flow {
            Ok(v)
        } else {
            Ok(Value::None)
        }
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

    fn assign_variable(&mut self, name: &str, value: Value) {
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

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        if self.flow != Flow::Next {
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
                    self.execute_stmts(then_branch)?;
                } else if let Some(else_stmts) = else_branch {
                    self.execute_stmts(else_stmts)?;
                }
            }
            Stmt::Return(expr) => {
                let val = expr
                    .as_ref()
                    .map_or(Ok(Value::None), |e| self.evaluate(e))?;
                self.flow = Flow::Return(val);
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
                    self.assign_variable(ident, item.clone());
                    self.execute_stmts(body)?;
                    match &self.flow {
                        Flow::Break => {
                            self.flow = Flow::Next;
                            break;
                        }
                        Flow::Continue => {
                            self.flow = Flow::Next;
                            continue;
                        }
                        Flow::Return(_) => return Ok(()),
                        Flow::Next => {}
                    }
                }
            }
            Stmt::Break => self.flow = Flow::Break,
            Stmt::Continue => self.flow = Flow::Continue,
        }
        Ok(())
    }

    fn execute_stmts(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            self.execute(stmt)?;
            if self.flow != Flow::Next {
                break;
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Identifier(name) => self.lookup_variable(name),
            Expr::BinaryOp(left, op, right) => self.apply_binary_op(left, op, right),
            Expr::UnaryOp(op, right) => self.apply_unary_op(op, right),
            Expr::LogicalOp(left, op, right) => self.apply_logical_op(left, op, right),
            Expr::Call(callee, args) => self.call_function(callee, args),
            Expr::List(elements) => self.evaluate_list_literal(elements),
            Expr::Tuple(elements) => self.evaluate_tuple_literal(elements),
            Expr::Dictionary(entries) => self.evaluate_dict_literal(entries),
            Expr::Index(obj, index) => self.evaluate_index(obj, index),
            Expr::GetAttr(obj, name) => self.evaluate_getattr(obj, name.to_string()),
            Expr::Slice(obj, start, stop, step) => self.evaluate_slice(obj, start, stop, step),
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

    fn evaluate_tuple_literal(&mut self, elements: &[Expr]) -> Result<Value, String> {
        let mut vals = Vec::new();
        for expr in elements {
            vals.push(self.evaluate(expr)?);
        }
        Ok(Value::Tuple(vals))
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
                let true_idx = if idx_int < 0 {
                    list.len() as i64 + idx_int
                } else {
                    idx_int
                };
                if true_idx < 0 || true_idx as usize >= list.len() {
                    return Err("List index out of range".to_string());
                }
                Ok(list[true_idx as usize].clone())
            }
            Value::Tuple(t) => {
                let idx_int = match idx_val {
                    Value::Int(i) => i,
                    _ => return Err("Tuple indices must be integers".to_string()),
                };
                let true_idx = if idx_int < 0 {
                    t.len() as i64 + idx_int
                } else {
                    idx_int
                };
                if true_idx < 0 || true_idx as usize >= t.len() {
                    return Err("Tuple index out of range".to_string());
                }
                Ok(t[true_idx as usize].clone())
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

    fn evaluate_slice(
        &mut self,
        obj: &Expr,
        start: &Option<Box<Expr>>,
        stop: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
    ) -> Result<Value, String> {
        let obj_val = self.evaluate(obj)?;
        match obj_val {
            Value::List(l) => {
                let list = l.borrow();
                let len = list.len();
                let step_val = if let Some(s) = step {
                    match self.evaluate(s)? {
                        Value::Int(i) => i,
                        _ => return Err("Slice step must be integer".into()),
                    }
                } else {
                    1
                };
                if step_val == 0 {
                    return Err("slice step cannot be zero".into());
                }

                let start_val = if let Some(s) = start {
                    match self.evaluate(s)? {
                        Value::Int(i) => i,
                        _ => return Err("Slice start must be integer".into()),
                    }
                } else {
                    if step_val > 0 {
                        0
                    } else {
                        len as i64 - 1
                    }
                };
                let stop_val = if let Some(s) = stop {
                    match self.evaluate(s)? {
                        Value::Int(i) => i,
                        _ => return Err("Slice stop must be integer".into()),
                    }
                } else {
                    if step_val > 0 {
                        len as i64
                    } else {
                        -1
                    }
                };

                let mut result = Vec::new();
                let mut current = normalize_index(start_val, len) as i64;
                let end = normalize_index(stop_val, len) as i64;

                if step_val > 0 {
                    while current < end && current < len as i64 {
                        result.push(list[current as usize].clone());
                        current += step_val;
                    }
                } else {
                    // Very basic backwards loop for now
                    while current > end && current >= 0 && current < len as i64 {
                        result.push(list[current as usize].clone());
                        current += step_val;
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }
            _ => Err(format!("Type not sliceable: {:?}", obj_val)),
        }
    }

    fn evaluate_getattr(&mut self, obj: &Expr, name: String) -> Result<Value, String> {
        let obj_val = self.evaluate(obj)?;
        Ok(Value::BoundMethod(Box::new(obj_val), name))
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
                        "Function '{}' expected {} arguments.",
                        name,
                        params.len()
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
                let old_flow = self.flow.clone();
                self.flow = Flow::Next;

                self.execute_stmts(&body)?;

                let ret_val = if let Flow::Return(v) = &self.flow {
                    v.clone()
                } else {
                    Value::None
                };
                self.env = original_env;
                self.flow = old_flow;
                Ok(ret_val)
            }
            Value::BoundMethod(receiver, method_name) => {
                self.call_bound_method(&receiver, &method_name, args_slice)
            }
            _ => Err(format!("Cannot call value of type: {:?}", callee_val)),
        }
    }

    fn call_bound_method(
        &mut self,
        receiver: &Value,
        method: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        match (receiver, method) {
            (Value::List(l), "append") => {
                if args.len() != 1 {
                    return Err("append() takes exactly one argument".into());
                }
                l.borrow_mut().push(args[0].clone());
                Ok(Value::None)
            }
            (Value::List(l), "pop") => {
                if let Some(v) = l.borrow_mut().pop() {
                    Ok(v)
                } else {
                    Err("pop from empty list".into())
                }
            }
            (Value::Dictionary(d), "keys") => {
                let keys: Vec<Value> = d
                    .borrow()
                    .keys()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(keys))))
            }
            (Value::String(s), "split") => {
                let delim = if args.len() > 0 {
                    args[0].to_string()
                } else {
                    " ".to_string()
                };
                let parts: Vec<Value> = s
                    .split(&delim)
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(parts))))
            }
            (Value::String(s), "strip") => Ok(Value::String(s.trim().to_string())),
            (Value::String(s), "lower") => Ok(Value::String(s.to_lowercase())),
            (Value::String(s), "upper") => Ok(Value::String(s.to_uppercase())),
            _ => Err(format!(
                "Object of type '{}' has no method '{}'",
                get_type_name(receiver),
                method
            )),
        }
    }

    fn apply_unary_op(&mut self, op: &Token, right: &Expr) -> Result<Value, String> {
        let val = self.evaluate(right)?;
        match op {
            Token::Minus => match val {
                Value::Int(i) => Ok(Value::Int(-i)),
                _ => Err("Unary '-' only valid for integers".into()),
            },
            Token::Not => Ok(Value::Bool(!is_truthy(&val))),
            Token::BitNot => match val {
                Value::Int(i) => Ok(Value::Int(!i)),
                _ => Err("Bitwise '~' only valid for integers".into()),
            },
            _ => Err("Invalid unary operator".into()),
        }
    }

    fn apply_logical_op(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Value, String> {
        let left_val = self.evaluate(left)?;
        match op {
            Token::Or => {
                if is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            Token::And => {
                if !is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            _ => Err("Invalid logical operator".into()),
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
            (Value::Int(a), Token::LtEq, Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Token::GtEq, Value::Int(b)) => Ok(Value::Bool(a >= b)),

            (Value::String(a), Token::Lt, Value::String(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), Token::Gt, Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Token::LtEq, Value::String(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), Token::GtEq, Value::String(b)) => Ok(Value::Bool(a >= b)),

            (Value::Int(a), Token::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Int(a), Token::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Int(a), Token::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Int(a), Token::Slash, Value::Int(b)) => {
                if b == 0 {
                    return Err("divide by zero".into());
                }
                Ok(Value::Int(a / b))
            }

            // Bitwise
            (Value::Int(a), Token::BitAnd, Value::Int(b)) => Ok(Value::Int(a & b)),
            (Value::Int(a), Token::BitOr, Value::Int(b)) => Ok(Value::Int(a | b)),
            (Value::Int(a), Token::BitXor, Value::Int(b)) => Ok(Value::Int(a ^ b)),
            (Value::Int(a), Token::LShift, Value::Int(b)) => Ok(Value::Int(a << b)),
            (Value::Int(a), Token::RShift, Value::Int(b)) => Ok(Value::Int(a >> b)),

            (Value::String(a), Token::Plus, Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(format!(
                "Unsupported binary op: {:?} {:?} {:?}",
                self.evaluate(left)?,
                op,
                self.evaluate(right)?
            )),
        }
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
            Value::Tuple(t) => format!(
                "({})",
                t.iter()
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
            Value::BoundMethod(receiver, name) => {
                format!("<bound method {}.{} >", get_type_name(receiver), name)
            }
        }
    }
}
