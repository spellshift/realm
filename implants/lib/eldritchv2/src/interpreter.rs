use super::ast::{
    Argument, BuiltinFn, Environment, Expr, FStringSegment, Function, Param, RuntimeParam, Stmt,
    Value,
};
use super::lexer::Lexer;
use super::parser::Parser;
use super::token::Token;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::Ordering;

// Reduced to prevent stack overflow in debug builds
const MAX_RECURSION_DEPTH: usize = 64;

// --- Helper Functions ---

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::None => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::String(s) => !s.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
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
        Value::Bytes(_) => "bytes".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Dictionary(_) => "dict".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Function(_) | Value::NativeFunction(_, _) | Value::BoundMethod(_, _) => {
            "function".to_string()
        }
    }
}

fn get_dir_attributes(value: &Value) -> Vec<String> {
    let mut attrs = match value {
        Value::List(_) => vec![
            "append".to_string(),
            "extend".to_string(),
            "index".to_string(),
            "insert".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "sort".to_string(),
        ],
        Value::Dictionary(_) => vec![
            "get".to_string(),
            "items".to_string(),
            "keys".to_string(),
            "popitem".to_string(),
            "update".to_string(),
            "values".to_string(),
        ],
        Value::String(_) => vec![
            "endswith".to_string(),
            "find".to_string(),
            "format".to_string(),
            "join".to_string(),
            "lower".to_string(),
            "replace".to_string(),
            "split".to_string(),
            "startswith".to_string(),
            "strip".to_string(),
            "upper".to_string(),
        ],
        _ => Vec::new(),
    };
    attrs.sort();
    attrs
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

// Helper for sorting
fn compare_values(a: &Value, b: &Value) -> Result<Ordering, String> {
    match (a, b) {
        (Value::None, Value::None) => Ok(Ordering::Equal),
        (Value::Bool(l), Value::Bool(r)) => Ok(l.cmp(r)),
        (Value::Int(l), Value::Int(r)) => Ok(l.cmp(r)),
        (Value::String(l), Value::String(r)) => Ok(l.cmp(r)),
        (Value::Bytes(l), Value::Bytes(r)) => Ok(l.cmp(r)),
        (Value::List(l), Value::List(r)) => {
            let l_vec = l.borrow();
            let r_vec = r.borrow();
            for (v1, v2) in l_vec.iter().zip(r_vec.iter()) {
                let ord = compare_values(v1, v2)?;
                if ord != Ordering::Equal {
                    return Ok(ord);
                }
            }
            Ok(l_vec.len().cmp(&r_vec.len()))
        }
        (Value::Tuple(l), Value::Tuple(r)) => {
            for (v1, v2) in l.iter().zip(r.iter()) {
                let ord = compare_values(v1, v2)?;
                if ord != Ordering::Equal {
                    return Ok(ord);
                }
            }
            Ok(l.len().cmp(&r.len()))
        }
        _ => Err(format!(
            "Type mismatch or unsortable types: {} <-> {}",
            get_type_name(a),
            get_type_name(b)
        )),
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
    pub depth: usize, // Tracks recursion depth
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

        interpreter.define_builtins();
        interpreter
    }

    fn define_builtin(&mut self, name: &str, _arity: usize, func: BuiltinFn) {
        self.env.borrow_mut().values.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), func),
        );
    }

    pub fn register_function(&mut self, name: &str, func: BuiltinFn) {
        self.define_builtin(name, 0, func);
    }

    fn define_builtins(&mut self) {
        self.define_builtin("print", 1, |args| {
            #[cfg(feature = "std")]
            println!("{}", args[0].to_string());
            Ok(Value::None)
        });

        self.define_builtin("len", 1, |args| match &args[0] {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
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

        self.define_builtin("enumerate", 1, |args| {
            let iterable = &args[0];
            let start = if args.len() > 1 {
                match args[1] {
                    Value::Int(i) => i,
                    _ => return Err("enumerate() start must be an integer".to_string()),
                }
            } else {
                0
            };

            let items = match iterable {
                Value::List(l) => l.borrow().clone(),
                Value::Tuple(t) => t.clone(),
                Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                _ => {
                    return Err(format!(
                        "Type '{:?}' is not iterable",
                        get_type_name(iterable)
                    ))
                }
            };

            let mut pairs = Vec::new();
            for (i, item) in items.into_iter().enumerate() {
                pairs.push(Value::Tuple(vec![Value::Int(i as i64 + start), item]));
            }

            Ok(Value::List(Rc::new(RefCell::new(pairs))))
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

        self.define_builtin("dir", 1, |args| {
            if args.is_empty() {
                let builtins = vec![
                    "assert",
                    "assert_eq",
                    "bool",
                    "dir",
                    "enumerate",
                    "fail",
                    "int",
                    "len",
                    "print",
                    "range",
                    "str",
                    "type",
                ];
                let val_attrs: Vec<Value> = builtins
                    .into_iter()
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                return Ok(Value::List(Rc::new(RefCell::new(val_attrs))));
            }
            let attrs = get_dir_attributes(&args[0]);
            let val_attrs: Vec<Value> = attrs.into_iter().map(Value::String).collect();
            Ok(Value::List(Rc::new(RefCell::new(val_attrs))))
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
            values: BTreeMap::new(),
        }));

        let mut eval_interp = Interpreter {
            env: eval_env,
            flow: Flow::Next,
            depth: self.depth,
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

    fn define_variable(&mut self, name: &str, value: Value) {
        self.env.borrow_mut().values.insert(name.to_string(), value);
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
                let mut runtime_params = Vec::new();
                for param in params {
                    match param {
                        Param::Normal(n) => runtime_params.push(RuntimeParam::Normal(n.clone())),
                        Param::Star(n) => runtime_params.push(RuntimeParam::Star(n.clone())),
                        Param::StarStar(n) => {
                            runtime_params.push(RuntimeParam::StarStar(n.clone()))
                        }
                        Param::WithDefault(n, default_expr) => {
                            let val = self.evaluate(default_expr)?;
                            runtime_params.push(RuntimeParam::WithDefault(n.clone(), val));
                        }
                    }
                }

                let func = Value::Function(Function {
                    name: name.clone(),
                    params: runtime_params,
                    body: body.clone(),
                    closure: self.env.clone(),
                });
                self.env.borrow_mut().values.insert(name.clone(), func);
            }
            Stmt::For(idents, iterable, body) => {
                let iterable_val = self.evaluate(iterable)?;
                let list_rc = match iterable_val {
                    Value::List(rc) => rc,
                    Value::Tuple(v) => Rc::new(RefCell::new(v)), // Temporary wrap for iterator
                    _ => {
                        return Err(format!(
                            "'for' loop can only iterate over lists/iterables. Found {:?}",
                            iterable_val
                        ))
                    }
                };
                let list = list_rc.borrow();

                for item in list.iter() {
                    if idents.len() == 1 {
                        self.assign_variable(&idents[0], item.clone());
                    } else {
                        // Unpacking
                        let parts = match item {
                            Value::List(l) => l.borrow().clone(),
                            Value::Tuple(t) => t.clone(),
                            _ => return Err("Cannot unpack non-iterable".to_string()),
                        };

                        if parts.len() != idents.len() {
                            return Err(format!("ValueError: too many/not enough values to unpack (expected {}, got {})", idents.len(), parts.len()));
                        }

                        for (var, val) in idents.iter().zip(parts.into_iter()) {
                            self.assign_variable(var, val);
                        }
                    }

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
            Expr::ListComp {
                body,
                var,
                iterable,
                cond,
            } => self.evaluate_list_comp(body, var, iterable, cond),
            Expr::DictComp {
                key,
                value,
                var,
                iterable,
                cond,
            } => self.evaluate_dict_comp(key, value, var, iterable, cond),
        }
    }

    // ... [Rest of file unchanged except for removal of Stmt::For with single variable] ...

    fn evaluate_list_comp(
        &mut self,
        body: &Expr,
        var: &str,
        iterable: &Expr,
        cond: &Option<Box<Expr>>,
    ) -> Result<Value, String> {
        let iterable_val = self.evaluate(iterable)?;
        let items = match iterable_val {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            _ => {
                return Err(format!(
                    "Type '{:?}' is not iterable",
                    get_type_name(&iterable_val)
                ))
            }
        };
        let comp_env = Rc::new(RefCell::new(Environment {
            parent: Some(Rc::clone(&self.env)),
            values: BTreeMap::new(),
        }));
        let original_env = Rc::clone(&self.env);
        self.env = comp_env;
        let mut results = Vec::new();
        for item in items {
            self.define_variable(var, item);
            let include = match cond {
                Some(c) => is_truthy(&self.evaluate(c)?),
                None => true,
            };
            if include {
                results.push(self.evaluate(body)?);
            }
        }
        self.env = original_env;
        Ok(Value::List(Rc::new(RefCell::new(results))))
    }

    fn evaluate_dict_comp(
        &mut self,
        key_expr: &Expr,
        val_expr: &Expr,
        var: &str,
        iterable: &Expr,
        cond: &Option<Box<Expr>>,
    ) -> Result<Value, String> {
        let iterable_val = self.evaluate(iterable)?;
        let items = match iterable_val {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            _ => {
                return Err(format!(
                    "Type '{:?}' is not iterable",
                    get_type_name(&iterable_val)
                ))
            }
        };
        let comp_env = Rc::new(RefCell::new(Environment {
            parent: Some(Rc::clone(&self.env)),
            values: BTreeMap::new(),
        }));
        let original_env = Rc::clone(&self.env);
        self.env = comp_env;
        let mut results = BTreeMap::new();
        for item in items {
            self.define_variable(var, item);
            let include = match cond {
                Some(c) => is_truthy(&self.evaluate(c)?),
                None => true,
            };
            if include {
                let k = self.evaluate(key_expr)?;
                let v = self.evaluate(val_expr)?;
                let k_str = match k {
                    Value::String(s) => s,
                    _ => return Err("Dict keys must be strings".into()),
                };
                results.insert(k_str, v);
            }
        }
        self.env = original_env;
        Ok(Value::Dictionary(Rc::new(RefCell::new(results))))
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
        let mut map = BTreeMap::new();
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

    fn adjust_slice_indices(
        &self,
        length: i64,
        start: &Option<i64>,
        stop: &Option<i64>,
        step: i64,
    ) -> (i64, i64) {
        let start_val = if let Some(s) = start {
            let mut s = *s;
            if s < 0 {
                s += length;
            }
            if step < 0 {
                if s >= length {
                    length - 1
                } else if s < 0 {
                    -1
                } else {
                    s
                }
            } else {
                if s < 0 {
                    0
                } else if s > length {
                    length
                } else {
                    s
                }
            }
        } else {
            if step < 0 {
                length - 1
            } else {
                0
            }
        };

        let stop_val = if let Some(s) = stop {
            let mut s = *s;
            if s < 0 {
                s += length;
            }
            if step < 0 {
                if s < -1 {
                    -1
                } else if s >= length {
                    length - 1
                } else {
                    s
                }
            } else {
                if s < 0 {
                    0
                } else if s > length {
                    length
                } else {
                    s
                }
            }
        } else {
            if step < 0 {
                -1
            } else {
                length
            }
        };

        (start_val, stop_val)
    }

    fn evaluate_slice(
        &mut self,
        obj: &Expr,
        start: &Option<Box<Expr>>,
        stop: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
    ) -> Result<Value, String> {
        let obj_val = self.evaluate(obj)?;

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

        let start_val_opt = if let Some(s) = start {
            match self.evaluate(s)? {
                Value::Int(i) => Some(i),
                _ => return Err("Slice start must be integer".into()),
            }
        } else {
            None
        };

        let stop_val_opt = if let Some(s) = stop {
            match self.evaluate(s)? {
                Value::Int(i) => Some(i),
                _ => return Err("Slice stop must be integer".into()),
            }
        } else {
            None
        };

        match obj_val {
            Value::List(l) => {
                let list = l.borrow();
                let len = list.len() as i64;
                let (i, j) =
                    self.adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);

                let mut result = Vec::new();
                let mut curr = i;

                if step_val > 0 {
                    while curr < j {
                        if curr >= 0 && curr < len {
                            result.push(list[curr as usize].clone());
                        }
                        curr += step_val;
                    }
                } else {
                    while curr > j {
                        if curr >= 0 && curr < len {
                            result.push(list[curr as usize].clone());
                        }
                        curr += step_val;
                    }
                }
                Ok(Value::List(Rc::new(RefCell::new(result))))
            }
            Value::Tuple(t) => {
                let len = t.len() as i64;
                let (i, j) =
                    self.adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
                let mut result = Vec::new();
                let mut curr = i;
                if step_val > 0 {
                    while curr < j {
                        if curr >= 0 && curr < len {
                            result.push(t[curr as usize].clone());
                        }
                        curr += step_val;
                    }
                } else {
                    while curr > j {
                        if curr >= 0 && curr < len {
                            result.push(t[curr as usize].clone());
                        }
                        curr += step_val;
                    }
                }
                Ok(Value::Tuple(result))
            }
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len() as i64;
                let (i, j) =
                    self.adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
                let mut result_chars = Vec::new();
                let mut curr = i;
                if step_val > 0 {
                    while curr < j {
                        if curr >= 0 && curr < len {
                            result_chars.push(chars[curr as usize]);
                        }
                        curr += step_val;
                    }
                } else {
                    while curr > j {
                        if curr >= 0 && curr < len {
                            result_chars.push(chars[curr as usize]);
                        }
                        curr += step_val;
                    }
                }
                Ok(Value::String(result_chars.into_iter().collect()))
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

    fn call_function(&mut self, callee: &Expr, args: &[Argument]) -> Result<Value, String> {
        let callee_val = self.evaluate(callee)?;

        let mut pos_args_val = Vec::new();
        let mut kw_args_val = BTreeMap::new();

        for arg in args {
            match arg {
                Argument::Positional(expr) => pos_args_val.push(self.evaluate(expr)?),
                Argument::Keyword(name, expr) => {
                    let val = self.evaluate(expr)?;
                    kw_args_val.insert(name.clone(), val);
                }
                Argument::StarArgs(expr) => {
                    let val = self.evaluate(expr)?;
                    match val {
                        Value::List(l) => pos_args_val.extend(l.borrow().clone()),
                        Value::Tuple(t) => pos_args_val.extend(t.clone()),
                        _ => {
                            return Err(format!(
                                "*args argument must be iterable, got {:?}",
                                get_type_name(&val)
                            ))
                        }
                    }
                }
                Argument::KwArgs(expr) => {
                    let val = self.evaluate(expr)?;
                    match val {
                        Value::Dictionary(d) => kw_args_val.extend(d.borrow().clone()),
                        _ => {
                            return Err(format!(
                                "**kwargs argument must be a dict, got {:?}",
                                get_type_name(&val)
                            ))
                        }
                    }
                }
            }
        }

        let args_slice = pos_args_val.as_slice();

        match callee_val {
            Value::NativeFunction(_, f) => f(args_slice),
            Value::Function(Function {
                name,
                params,
                body,
                closure,
            }) => {
                if self.depth >= MAX_RECURSION_DEPTH {
                    return Err("Recursion limit exceeded".into());
                }
                self.depth += 1;

                let result = (|| {
                    let function_env = Rc::new(RefCell::new(Environment {
                        parent: Some(closure),
                        values: BTreeMap::new(),
                    }));

                    let mut pos_idx = 0;

                    for param in params {
                        match param {
                            RuntimeParam::Normal(param_name) => {
                                if pos_idx < pos_args_val.len() {
                                    function_env
                                        .borrow_mut()
                                        .values
                                        .insert(param_name.clone(), pos_args_val[pos_idx].clone());
                                    pos_idx += 1;
                                } else if let Some(val) = kw_args_val.remove(&param_name) {
                                    function_env
                                        .borrow_mut()
                                        .values
                                        .insert(param_name.clone(), val);
                                } else {
                                    return Err(format!(
                                        "Missing required argument: '{}'",
                                        param_name
                                    ));
                                }
                            }
                            RuntimeParam::WithDefault(param_name, default_val) => {
                                if pos_idx < pos_args_val.len() {
                                    function_env
                                        .borrow_mut()
                                        .values
                                        .insert(param_name.clone(), pos_args_val[pos_idx].clone());
                                    pos_idx += 1;
                                } else if let Some(val) = kw_args_val.remove(&param_name) {
                                    function_env
                                        .borrow_mut()
                                        .values
                                        .insert(param_name.clone(), val);
                                } else {
                                    function_env
                                        .borrow_mut()
                                        .values
                                        .insert(param_name.clone(), default_val.clone());
                                }
                            }
                            RuntimeParam::Star(param_name) => {
                                let remaining = if pos_idx < pos_args_val.len() {
                                    pos_args_val[pos_idx..].to_vec()
                                } else {
                                    Vec::new()
                                };
                                pos_idx = pos_args_val.len();
                                function_env
                                    .borrow_mut()
                                    .values
                                    .insert(param_name.clone(), Value::Tuple(remaining));
                            }
                            RuntimeParam::StarStar(param_name) => {
                                let mut dict = BTreeMap::new();
                                let keys_to_move: Vec<String> =
                                    kw_args_val.keys().cloned().collect();
                                for k in keys_to_move {
                                    if let Some(v) = kw_args_val.remove(&k) {
                                        dict.insert(k, v);
                                    }
                                }
                                function_env.borrow_mut().values.insert(
                                    param_name.clone(),
                                    Value::Dictionary(Rc::new(RefCell::new(dict))),
                                );
                            }
                        }
                    }

                    if pos_idx < pos_args_val.len() {
                        return Err(format!(
                            "Function '{}' got too many positional arguments.",
                            name
                        ));
                    }
                    if !kw_args_val.is_empty() {
                        let keys: Vec<&String> = kw_args_val.keys().collect();
                        return Err(format!(
                            "Function '{}' got unexpected keyword arguments: {:?}",
                            name, keys
                        ));
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
                })();

                self.depth -= 1;
                result
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
            (Value::List(l), "extend") => {
                if args.len() != 1 {
                    return Err("extend() takes exactly one argument".into());
                }
                let iterable = &args[0];
                match iterable {
                    Value::List(other) => l.borrow_mut().extend(other.borrow().clone()),
                    Value::Tuple(other) => l.borrow_mut().extend(other.clone()),
                    _ => {
                        return Err(format!(
                            "extend() expects an iterable, got {}",
                            get_type_name(iterable)
                        ))
                    }
                }
                Ok(Value::None)
            }
            (Value::List(l), "insert") => {
                if args.len() != 2 {
                    return Err("insert() takes exactly two arguments".into());
                }
                let idx = match args[0] {
                    Value::Int(i) => i,
                    _ => return Err("insert() index must be an integer".into()),
                };
                let val = args[1].clone();
                let mut vec = l.borrow_mut();
                let len = vec.len() as i64;
                let index = if idx < 0 {
                    (len + idx).max(0) as usize
                } else {
                    idx.min(len) as usize
                };
                vec.insert(index, val);
                Ok(Value::None)
            }
            (Value::List(l), "remove") => {
                if args.len() != 1 {
                    return Err("remove() takes exactly one argument".into());
                }
                let target = &args[0];
                let mut vec = l.borrow_mut();
                if let Some(pos) = vec.iter().position(|x| x == target) {
                    vec.remove(pos);
                    Ok(Value::None)
                } else {
                    Err("ValueError: list.remove(x): x not in list".into())
                }
            }
            (Value::List(l), "index") => {
                if args.len() != 1 {
                    return Err("index() takes exactly one argument".into());
                } // Simplification
                let target = &args[0];
                let vec = l.borrow();
                if let Some(pos) = vec.iter().position(|x| x == target) {
                    Ok(Value::Int(pos as i64))
                } else {
                    Err("ValueError: list.index(x): x not in list".into())
                }
            }
            (Value::List(l), "pop") => {
                if let Some(v) = l.borrow_mut().pop() {
                    Ok(v)
                } else {
                    Err("pop from empty list".into())
                }
            }
            (Value::List(l), "sort") => {
                let mut vec = l.borrow_mut();
                vec.sort_by(|a, b| compare_values(a, b).unwrap_or(Ordering::Equal));
                Ok(Value::None)
            }

            (Value::Dictionary(d), "keys") => {
                let keys: Vec<Value> = d
                    .borrow()
                    .keys()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(keys))))
            }
            (Value::Dictionary(d), "values") => {
                let values: Vec<Value> = d.borrow().values().cloned().collect();
                Ok(Value::List(Rc::new(RefCell::new(values))))
            }
            (Value::Dictionary(d), "items") => {
                let items: Vec<Value> = d
                    .borrow()
                    .iter()
                    .map(|(k, v)| Value::Tuple(vec![Value::String(k.clone()), v.clone()]))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(items))))
            }
            (Value::Dictionary(d), "get") => {
                if args.len() < 1 || args.len() > 2 {
                    return Err("get() takes 1 or 2 arguments".into());
                }
                let key = match &args[0] {
                    Value::String(s) => s,
                    _ => return Err("Dict keys must be strings".into()),
                };
                let default = if args.len() == 2 {
                    args[1].clone()
                } else {
                    Value::None
                };
                match d.borrow().get(key) {
                    Some(v) => Ok(v.clone()),
                    None => Ok(default),
                }
            }
            (Value::Dictionary(d), "update") => {
                if args.len() != 1 {
                    return Err("update() takes exactly one argument".into());
                }
                match &args[0] {
                    Value::Dictionary(other) => {
                        let other_map = other.borrow().clone();
                        d.borrow_mut().extend(other_map);
                        Ok(Value::None)
                    }
                    _ => Err("update() requires a dictionary".into()),
                }
            }
            (Value::Dictionary(d), "popitem") => {
                let mut map = d.borrow_mut();
                let last_key = map.keys().next_back().cloned();
                if let Some(k) = last_key {
                    let v = map.remove(&k).unwrap();
                    Ok(Value::Tuple(vec![Value::String(k), v]))
                } else {
                    Err("popitem(): dictionary is empty".into())
                }
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
            (Value::String(s), "startswith") => {
                if args.len() != 1 {
                    return Err("startswith() takes 1 argument".into());
                }
                let prefix = args[0].to_string();
                Ok(Value::Bool(s.starts_with(&prefix)))
            }
            (Value::String(s), "endswith") => {
                if args.len() != 1 {
                    return Err("endswith() takes 1 argument".into());
                }
                let suffix = args[0].to_string();
                Ok(Value::Bool(s.ends_with(&suffix)))
            }
            (Value::String(s), "find") => {
                if args.len() != 1 {
                    return Err("find() takes 1 argument".into());
                }
                let sub = args[0].to_string();
                match s.find(&sub) {
                    Some(idx) => Ok(Value::Int(idx as i64)),
                    None => Ok(Value::Int(-1)),
                }
            }
            (Value::String(s), "replace") => {
                if args.len() != 2 {
                    return Err("replace() takes 2 arguments".into());
                }
                let old = args[0].to_string();
                let new = args[1].to_string();
                Ok(Value::String(s.replace(&old, &new)))
            }
            (Value::String(s), "join") => {
                if args.len() != 1 {
                    return Err("join() takes 1 argument".into());
                }
                match &args[0] {
                    Value::List(l) => {
                        let list = l.borrow();
                        let strs: Result<Vec<String>, _> = list
                            .iter()
                            .map(|v| match v {
                                Value::String(ss) => Ok(ss.clone()),
                                _ => Err("join() expects list of strings"),
                            })
                            .collect();
                        Ok(Value::String(strs?.join(s)))
                    }
                    _ => Err("join() expects a list".into()),
                }
            }
            (Value::String(s), "format") => {
                let mut result = String::new();
                let mut arg_idx = 0;
                let chars: Vec<char> = s.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '}' {
                        if arg_idx >= args.len() {
                            return Err("tuple index out of range".into());
                        }
                        result.push_str(&args[arg_idx].to_string());
                        arg_idx += 1;
                        i += 2;
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                Ok(Value::String(result))
            }

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

            // INT Comparisons
            (Value::Int(a), Token::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Token::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Token::LtEq, Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Token::GtEq, Value::Int(b)) => Ok(Value::Bool(a >= b)),

            // STRING Comparisons
            (Value::String(a), Token::Lt, Value::String(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), Token::Gt, Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), Token::LtEq, Value::String(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), Token::GtEq, Value::String(b)) => Ok(Value::Bool(a >= b)),

            // Arithmetic
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
            Value::Bytes(b) => format!("{:?}", b),
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
