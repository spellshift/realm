mod builtins;
mod error;
mod methods;
mod utils;

use crate::ast::{
    Argument, BuiltinFn, Environment, Expr, ExprKind, FStringSegment, Function, Param,
    RuntimeParam, Stmt, StmtKind, Value,
};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::{Span, TokenKind};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use error::runtime_error;
pub use error::EldritchError;
// Removed unused normalize_index
use builtins::get_all_builtins;
use methods::call_bound_method;
use utils::{adjust_slice_indices, get_type_name, is_truthy};

const MAX_RECURSION_DEPTH: usize = 64;

#[derive(Clone, PartialEq)]
pub enum Flow {
    Next,
    Break,
    Continue,
    Return(Value),
}
// ... (rest of Interpreter struct and implementation is the same as before)
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
        interpreter
    }

    fn load_builtins(&mut self) {
        for (name, func) in get_all_builtins() {
            self.register_function(name, func);
        }
        // Hardcoded pass variable for now
        self.env
            .borrow_mut()
            .values
            .insert("pass".to_string(), Value::None);
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

    // ... (rest of file omitted for brevity, it matches previous versions)
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
                StmtKind::Expression(expr) => {
                    last_val = self
                        .evaluate(expr)
                        .map_err(|e| self.format_error(input, e))?;
                }
                _ => {
                    self.execute(&stmt)
                        .map_err(|e| self.format_error(input, e))?;
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

    fn format_error(&self, source: &str, error: EldritchError) -> String {
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

    fn execute(&mut self, stmt: &Stmt) -> Result<(), EldritchError> {
        if self.flow != Flow::Next {
            return Ok(());
        }

        match &stmt.kind {
            StmtKind::Expression(expr) => {
                self.evaluate(expr)?;
            }
            StmtKind::Assignment(target_expr, value_expr) => {
                let value = self.evaluate(value_expr)?;
                self.assign(target_expr, value)?;
            }
            StmtKind::AugmentedAssignment(target_expr, op, value_expr) => {
                self.execute_augmented_assignment(target_expr, op, value_expr)?;
            }
            StmtKind::If(condition, then_branch, else_branch) => {
                let eval_cond = &self.evaluate(condition)?;
                if is_truthy(eval_cond) {
                    self.execute_stmts(then_branch)?;
                } else if let Some(else_stmts) = else_branch {
                    self.execute_stmts(else_stmts)?;
                }
            }
            StmtKind::Return(expr) => {
                let val = expr
                    .as_ref()
                    .map_or(Ok(Value::None), |e| self.evaluate(e))?;
                self.flow = Flow::Return(val);
            }
            StmtKind::Def(name, params, body) => {
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
            StmtKind::For(idents, iterable, body) => {
                let iterable_val = self.evaluate(iterable)?;
                let items: Vec<Value> = match iterable_val {
                    Value::List(l) => l.borrow().clone(),
                    Value::Tuple(t) => t.clone(),
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    Value::Bytes(b) => b.iter().map(|&byte| Value::Int(byte as i64)).collect(),
                    _ => {
                        return runtime_error(
                            iterable.span,
                            &format!(
                                "'for' loop can only iterate over lists/iterables. Found {:?}",
                                iterable_val
                            ),
                        )
                    }
                };

                for item in items {
                    if idents.len() == 1 {
                        self.assign_variable(&idents[0], item);
                    } else {
                        let parts = match item {
                            Value::List(l) => l.borrow().clone(),
                            Value::Tuple(t) => t.clone(),
                            _ => return runtime_error(stmt.span, "Cannot unpack non-iterable"),
                        };

                        if parts.len() != idents.len() {
                            return runtime_error(stmt.span, &format!("ValueError: too many/not enough values to unpack (expected {}, got {})", idents.len(), parts.len()));
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
            StmtKind::Break => self.flow = Flow::Break,
            StmtKind::Continue => self.flow = Flow::Continue,
            StmtKind::Pass => {} // Do nothing
        }
        Ok(())
    }

    fn execute_stmts(&mut self, stmts: &[Stmt]) -> Result<(), EldritchError> {
        for stmt in stmts {
            self.execute(stmt)?;
            if self.flow != Flow::Next {
                break;
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, EldritchError> {
        let span = expr.span;
        match &expr.kind {
            ExprKind::Literal(value) => Ok(value.clone()),
            ExprKind::Identifier(name) => self.lookup_variable(name, span),
            ExprKind::BinaryOp(left, op, right) => self.apply_binary_op(left, op, right, span),
            ExprKind::UnaryOp(op, right) => self.apply_unary_op(op, right, span),
            ExprKind::LogicalOp(left, op, right) => self.apply_logical_op(left, op, right, span),
            ExprKind::Call(callee, args) => self.call_function(callee, args, span),
            ExprKind::List(elements) => self.evaluate_list_literal(elements),
            ExprKind::Tuple(elements) => self.evaluate_tuple_literal(elements),
            ExprKind::Dictionary(entries) => self.evaluate_dict_literal(entries),
            ExprKind::Index(obj, index) => self.evaluate_index(obj, index, span),
            ExprKind::GetAttr(obj, name) => self.evaluate_getattr(obj, name.to_string()),
            ExprKind::Slice(obj, start, stop, step) => {
                self.evaluate_slice(obj, start, stop, step, span)
            }
            ExprKind::FString(segments) => self.evaluate_fstring(segments),
            ExprKind::ListComp {
                body,
                var,
                iterable,
                cond,
            } => self.evaluate_list_comp(body, var, iterable, cond),
            ExprKind::DictComp {
                key,
                value,
                var,
                iterable,
                cond,
            } => self.evaluate_dict_comp(key, value, var, iterable, cond),
            ExprKind::Lambda { params, body } => self.evaluate_lambda(params, body),
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate(cond)?;
                if is_truthy(&cond_val) {
                    self.evaluate(then_branch)
                } else {
                    self.evaluate(else_branch)
                }
            }
        }
    }

    fn evaluate_lambda(
        &mut self,
        params: &Vec<Param>,
        body: &Expr,
    ) -> Result<Value, EldritchError> {
        let mut runtime_params = Vec::new();
        for param in params {
            match param {
                Param::Normal(n) => runtime_params.push(RuntimeParam::Normal(n.clone())),
                Param::Star(n) => runtime_params.push(RuntimeParam::Star(n.clone())),
                Param::StarStar(n) => runtime_params.push(RuntimeParam::StarStar(n.clone())),
                Param::WithDefault(n, default_expr) => {
                    let val = self.evaluate(default_expr)?;
                    runtime_params.push(RuntimeParam::WithDefault(n.clone(), val));
                }
            }
        }
        let ret_stmt = Stmt {
            kind: StmtKind::Return(Some(body.clone())),
            span: body.span,
        };

        let func = Value::Function(Function {
            name: "<lambda>".to_string(),
            params: runtime_params,
            body: vec![ret_stmt],
            closure: self.env.clone(),
        });
        Ok(func)
    }

    fn evaluate_list_comp(
        &mut self,
        body: &Expr,
        var: &str,
        iterable: &Expr,
        cond: &Option<Box<Expr>>,
    ) -> Result<Value, EldritchError> {
        let iterable_val = self.evaluate(iterable)?;
        let items = match iterable_val {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            _ => {
                return runtime_error(
                    iterable.span,
                    &format!("Type '{:?}' is not iterable", get_type_name(&iterable_val)),
                )
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
    ) -> Result<Value, EldritchError> {
        let iterable_val = self.evaluate(iterable)?;
        let items = match iterable_val {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            _ => {
                return runtime_error(
                    iterable.span,
                    &format!("Type '{:?}' is not iterable", get_type_name(&iterable_val)),
                )
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
                    _ => return runtime_error(key_expr.span, "Dict keys must be strings"),
                };
                results.insert(k_str, v);
            }
        }
        self.env = original_env;
        Ok(Value::Dictionary(Rc::new(RefCell::new(results))))
    }

    fn evaluate_list_literal(&mut self, elements: &[Expr]) -> Result<Value, EldritchError> {
        let mut vals = Vec::new();
        for expr in elements {
            vals.push(self.evaluate(expr)?);
        }
        Ok(Value::List(Rc::new(RefCell::new(vals))))
    }

    fn evaluate_tuple_literal(&mut self, elements: &[Expr]) -> Result<Value, EldritchError> {
        let mut vals = Vec::new();
        for expr in elements {
            vals.push(self.evaluate(expr)?);
        }
        Ok(Value::Tuple(vals))
    }

    fn evaluate_dict_literal(&mut self, entries: &[(Expr, Expr)]) -> Result<Value, EldritchError> {
        let mut map = BTreeMap::new();
        for (key_expr, value_expr) in entries {
            let key_val = self.evaluate(key_expr)?;
            let value_val = self.evaluate(value_expr)?;
            let key_str = match key_val {
                Value::String(s) => s,
                _ => return runtime_error(key_expr.span, "Dictionary keys must be strings."),
            };
            map.insert(key_str, value_val);
        }
        Ok(Value::Dictionary(Rc::new(RefCell::new(map))))
    }

    fn evaluate_index(
        &mut self,
        obj: &Expr,
        index: &Expr,
        span: Span,
    ) -> Result<Value, EldritchError> {
        let obj_val = self.evaluate(obj)?;
        let idx_val = self.evaluate(index)?;

        match obj_val {
            Value::List(l) => {
                let idx_int = match idx_val {
                    Value::Int(i) => i,
                    _ => return runtime_error(index.span, "List indices must be integers"),
                };
                let list = l.borrow();
                let true_idx = if idx_int < 0 {
                    list.len() as i64 + idx_int
                } else {
                    idx_int
                };
                if true_idx < 0 || true_idx as usize >= list.len() {
                    return runtime_error(span, "List index out of range");
                }
                Ok(list[true_idx as usize].clone())
            }
            Value::Tuple(t) => {
                let idx_int = match idx_val {
                    Value::Int(i) => i,
                    _ => return runtime_error(index.span, "Tuple indices must be integers"),
                };
                let true_idx = if idx_int < 0 {
                    t.len() as i64 + idx_int
                } else {
                    idx_int
                };
                if true_idx < 0 || true_idx as usize >= t.len() {
                    return runtime_error(span, "Tuple index out of range");
                }
                Ok(t[true_idx as usize].clone())
            }
            Value::Dictionary(d) => {
                let key_str = match idx_val {
                    Value::String(s) => s,
                    _ => return runtime_error(index.span, "Dictionary keys must be strings"),
                };
                let dict = d.borrow();
                match dict.get(&key_str) {
                    Some(v) => Ok(v.clone()),
                    None => runtime_error(span, &format!("KeyError: '{}'", key_str)),
                }
            }
            _ => runtime_error(obj.span, &format!("Type not subscriptable: {:?}", obj_val)),
        }
    }

    fn evaluate_slice(
        &mut self,
        obj: &Expr,
        start: &Option<Box<Expr>>,
        stop: &Option<Box<Expr>>,
        step: &Option<Box<Expr>>,
        span: Span,
    ) -> Result<Value, EldritchError> {
        let obj_val = self.evaluate(obj)?;

        let step_val = if let Some(s) = step {
            match self.evaluate(s)? {
                Value::Int(i) => i,
                _ => return runtime_error(s.span, "Slice step must be integer"),
            }
        } else {
            1
        };

        if step_val == 0 {
            return runtime_error(span, "slice step cannot be zero");
        }

        let start_val_opt = if let Some(s) = start {
            match self.evaluate(s)? {
                Value::Int(i) => Some(i),
                _ => return runtime_error(s.span, "Slice start must be integer"),
            }
        } else {
            None
        };

        let stop_val_opt = if let Some(s) = stop {
            match self.evaluate(s)? {
                Value::Int(i) => Some(i),
                _ => return runtime_error(s.span, "Slice stop must be integer"),
            }
        } else {
            None
        };

        match obj_val {
            Value::List(l) => {
                let list = l.borrow();
                let len = list.len() as i64;
                let (i, j) =
                    utils::adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);

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
                    utils::adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
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
                    utils::adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
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
            _ => runtime_error(obj.span, &format!("Type not sliceable: {:?}", obj_val)),
        }
    }

    fn evaluate_getattr(&mut self, obj: &Expr, name: String) -> Result<Value, EldritchError> {
        let obj_val = self.evaluate(obj)?;

        // Support dot access for dictionary keys (useful for modules)
        if let Value::Dictionary(d) = &obj_val {
            if let Some(val) = d.borrow().get(&name) {
                return Ok(val.clone());
            }
        }

        Ok(Value::BoundMethod(Box::new(obj_val), name))
    }

    fn evaluate_fstring(&mut self, segments: &[FStringSegment]) -> Result<Value, EldritchError> {
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

    fn lookup_variable(&self, name: &str, span: Span) -> Result<Value, EldritchError> {
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

    fn call_function(
        &mut self,
        callee: &Expr,
        args: &[Argument],
        span: Span,
    ) -> Result<Value, EldritchError> {
        let callee_val = self.evaluate(callee)?;

        // Special handling for map/filter/reduce which take functions
        if let Value::NativeFunction(name, _) = &callee_val {
            if name == "map" {
                return self.builtin_map(args, span);
            } else if name == "filter" {
                return self.builtin_filter(args, span);
            } else if name == "reduce" {
                return self.builtin_reduce(args, span);
            }
        }

        // Standard call
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
                            return runtime_error(
                                expr.span,
                                &format!(
                                    "*args argument must be iterable, got {:?}",
                                    get_type_name(&val)
                                ),
                            )
                        }
                    }
                }
                Argument::KwArgs(expr) => {
                    let val = self.evaluate(expr)?;
                    match val {
                        Value::Dictionary(d) => kw_args_val.extend(d.borrow().clone()),
                        _ => {
                            return runtime_error(
                                expr.span,
                                &format!(
                                    "**kwargs argument must be a dict, got {:?}",
                                    get_type_name(&val)
                                ),
                            )
                        }
                    }
                }
            }
        }

        let args_slice = pos_args_val.as_slice();

        match callee_val {
            Value::NativeFunction(_, f) => {
                f(args_slice).map_err(|e| EldritchError { message: e, span })
            }
            Value::Function(Function {
                name,
                params,
                body,
                closure,
            }) => {
                #[allow(unused_variables)]
                let _ = name; // Silence unused name warning if any

                if self.depth >= MAX_RECURSION_DEPTH {
                    return runtime_error(span, "Recursion limit exceeded");
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
                                    return runtime_error(
                                        span,
                                        &format!("Missing required argument: '{}'", param_name),
                                    );
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
                        return runtime_error(span, "Function got too many positional arguments.");
                    }

                    if !kw_args_val.is_empty() {
                        let mut keys: Vec<&String> = kw_args_val.keys().collect();
                        keys.sort();
                        return runtime_error(
                            span,
                            &format!(
                                "Function '{}' got unexpected keyword arguments: {:?}",
                                name, keys
                            ),
                        );
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
                call_bound_method(&receiver, &method_name, args_slice)
                    .map_err(|e| EldritchError { message: e, span })
            }
            _ => runtime_error(
                span,
                &format!("Cannot call value of type: {:?}", callee_val),
            ),
        }
    }

    fn builtin_map(&mut self, args: &[Argument], span: Span) -> Result<Value, EldritchError> {
        if args.len() != 2 {
            return runtime_error(span, "map() takes exactly 2 arguments");
        }
        let func_val = self.evaluate_arg(&args[0])?;
        let iterable_val = self.evaluate_arg(&args[1])?;

        let items = self.to_iterable(&iterable_val, span)?;
        let mut results = Vec::new();

        for item in items {
            let res = self.call_value(&func_val, &[item], span)?;
            results.push(res);
        }

        Ok(Value::List(Rc::new(RefCell::new(results))))
    }

    fn builtin_filter(&mut self, args: &[Argument], span: Span) -> Result<Value, EldritchError> {
        if args.len() != 2 {
            return runtime_error(span, "filter() takes exactly 2 arguments");
        }
        let func_val = self.evaluate_arg(&args[0])?;
        let iterable_val = self.evaluate_arg(&args[1])?;
        let items = self.to_iterable(&iterable_val, span)?;

        let mut results = Vec::new();
        for item in items {
            let keep = if let Value::None = func_val {
                is_truthy(&item)
            } else {
                let res = self.call_value(&func_val, &[item.clone()], span)?;
                is_truthy(&res)
            };
            if keep {
                results.push(item);
            }
        }
        Ok(Value::List(Rc::new(RefCell::new(results))))
    }

    fn builtin_reduce(&mut self, args: &[Argument], span: Span) -> Result<Value, EldritchError> {
        if args.len() < 2 || args.len() > 3 {
            return runtime_error(span, "reduce() takes 2 or 3 arguments");
        }
        let func_val = self.evaluate_arg(&args[0])?;
        let iterable_val = self.evaluate_arg(&args[1])?;
        let mut items = self.to_iterable(&iterable_val, span)?.into_iter();

        let mut acc = if args.len() == 3 {
            self.evaluate_arg(&args[2])?
        } else {
            match items.next() {
                Some(v) => v,
                None => {
                    return runtime_error(span, "reduce() of empty sequence with no initial value")
                }
            }
        };

        for item in items {
            acc = self.call_value(&func_val, &[acc, item], span)?;
        }
        Ok(acc)
    }

    fn call_value(
        &mut self,
        func: &Value,
        args: &[Value],
        span: Span,
    ) -> Result<Value, EldritchError> {
        match func {
            Value::NativeFunction(_, f) => f(args).map_err(|e| EldritchError { message: e, span }),
            Value::Function(Function {
                name: _,
                params: _,
                body: _,
                closure: _,
            }) => {
                if self.depth >= MAX_RECURSION_DEPTH {
                    return runtime_error(span, "Recursion limit exceeded");
                }
                self.depth += 1;

                let expr_args: Vec<Argument> = args
                    .iter()
                    .map(|v| {
                        Argument::Positional(Expr {
                            kind: ExprKind::Literal(v.clone()),
                            span,
                        })
                    })
                    .collect();

                // Construct minimal callee expr for recursion call
                let callee_expr = Expr {
                    kind: ExprKind::Literal(func.clone()),
                    span,
                };

                let res = self.call_function(&callee_expr, &expr_args, span);
                self.depth -= 1;
                res
            }
            Value::BoundMethod(receiver, method_name) => {
                call_bound_method(receiver, method_name, args)
                    .map_err(|e| EldritchError { message: e, span })
            }
            _ => runtime_error(span, "not callable"),
        }
    }

    fn evaluate_arg(&mut self, arg: &Argument) -> Result<Value, EldritchError> {
        match arg {
            Argument::Positional(e) => self.evaluate(e),
            // Just return a dummy span here for the error, or match e.span if available.
            // Since we don't have easy access to a span here without unpacking, use a dummy one.
            _ => runtime_error(
                Span::new(0, 0, 0),
                "HOFs currently only support positional arguments",
            ),
        }
    }

    fn to_iterable(&self, val: &Value, span: Span) -> Result<Vec<Value>, EldritchError> {
        match val {
            Value::List(l) => Ok(l.borrow().clone()),
            Value::Tuple(t) => Ok(t.clone()),
            Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
            _ => runtime_error(
                span,
                &format!("Type '{:?}' is not iterable", get_type_name(val)),
            ),
        }
    }

    fn apply_unary_op(
        &mut self,
        op: &TokenKind,
        right: &Expr,
        span: Span,
    ) -> Result<Value, EldritchError> {
        let val = self.evaluate(right)?;
        match op {
            TokenKind::Minus => match val {
                Value::Int(i) => Ok(Value::Int(-i)),
                _ => runtime_error(span, "Unary '-' only valid for integers"),
            },
            TokenKind::Not => Ok(Value::Bool(!is_truthy(&val))),
            TokenKind::BitNot => match val {
                Value::Int(i) => Ok(Value::Int(!i)),
                _ => runtime_error(span, "Bitwise '~' only valid for integers"),
            },
            _ => runtime_error(span, "Invalid unary operator"),
        }
    }

    fn apply_logical_op(
        &mut self,
        left: &Expr,
        op: &TokenKind,
        right: &Expr,
        span: Span,
    ) -> Result<Value, EldritchError> {
        let left_val = self.evaluate(left)?;
        match op {
            TokenKind::Or => {
                if is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            TokenKind::And => {
                if !is_truthy(&left_val) {
                    return Ok(left_val);
                }
                self.evaluate(right)
            }
            _ => runtime_error(span, "Invalid logical operator"),
        }
    }

    fn evaluate_in(&mut self, item: &Value, collection: &Value, span: Span) -> Result<Value, EldritchError> {
        match collection {
            Value::List(l) => {
                let list = l.borrow();
                Ok(Value::Bool(list.contains(item)))
            }
            Value::Tuple(t) => Ok(Value::Bool(t.contains(item))),
            Value::Dictionary(d) => {
                let dict = d.borrow();
                // Check keys
                let key = match item {
                    Value::String(s) => s,
                    _ => return Ok(Value::Bool(false)), // Only strings are keys
                };
                Ok(Value::Bool(dict.contains_key(key)))
            }
            Value::String(s) => {
                let sub = match item {
                    Value::String(ss) => ss,
                    _ => return runtime_error(span, "'in <string>' requires string as left operand"),
                };
                Ok(Value::Bool(s.contains(sub)))
            }
            _ => runtime_error(span, &format!("argument of type '{}' is not iterable", get_type_name(collection))),
        }
    }

    fn apply_binary_op(
        &mut self,
        left: &Expr,
        op: &TokenKind,
        right: &Expr,
        span: Span,
    ) -> Result<Value, EldritchError> {
        let a = self.evaluate(left)?;
        let b = self.evaluate(right)?;

        match (a, op.clone(), b) {
            (a, TokenKind::Eq, b) => Ok(Value::Bool(a == b)),
            (a, TokenKind::NotEq, b) => Ok(Value::Bool(a != b)),

            // INT Comparisons
            (Value::Int(a), TokenKind::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), TokenKind::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), TokenKind::LtEq, Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), TokenKind::GtEq, Value::Int(b)) => Ok(Value::Bool(a >= b)),

            // STRING Comparisons
            (Value::String(a), TokenKind::Lt, Value::String(b)) => Ok(Value::Bool(a < b)),
            (Value::String(a), TokenKind::Gt, Value::String(b)) => Ok(Value::Bool(a > b)),
            (Value::String(a), TokenKind::LtEq, Value::String(b)) => Ok(Value::Bool(a <= b)),
            (Value::String(a), TokenKind::GtEq, Value::String(b)) => Ok(Value::Bool(a >= b)),

            // IN Operator
            (item, TokenKind::In, collection) => self.evaluate_in(&item, &collection, span),

            // Arithmetic
            (Value::Int(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Int(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Int(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Int(a), TokenKind::Slash, Value::Int(b)) => {
                if b == 0 {
                    return runtime_error(span, "divide by zero");
                }
                // Standard division (for integers, acts like floor in Rust, but behavior is technically floor div)
                Ok(Value::Int(a / b))
            }
            (Value::Int(a), TokenKind::SlashSlash, Value::Int(b)) => {
                if b == 0 {
                    return runtime_error(span, "divide by zero");
                }
                // Floor division with correct negative handling (Python style)
                // Rust integer division truncates toward zero.
                // We want floor (towards negative infinity).
                let mut res = a / b;
                // If the result is not exact and the signs are different, we need to subtract 1 (or add -1)
                if (a % b != 0) && ((a < 0) ^ (b < 0)) {
                    res -= 1;
                }
                Ok(Value::Int(res))
            }
            // Modulo
            (Value::Int(a), TokenKind::Percent, Value::Int(b)) => {
                if b == 0 {
                    return runtime_error(span, "modulo by zero");
                }
                // Python style modulo
                let res = ((a % b) + b) % b;
                Ok(Value::Int(res))
            }

            // Bitwise
            (Value::Int(a), TokenKind::BitAnd, Value::Int(b)) => Ok(Value::Int(a & b)),
            (Value::Int(a), TokenKind::BitOr, Value::Int(b)) => Ok(Value::Int(a | b)),
            (Value::Int(a), TokenKind::BitXor, Value::Int(b)) => Ok(Value::Int(a ^ b)),
            (Value::Int(a), TokenKind::LShift, Value::Int(b)) => Ok(Value::Int(a << b)),
            (Value::Int(a), TokenKind::RShift, Value::Int(b)) => Ok(Value::Int(a >> b)),

            (Value::String(a), TokenKind::Plus, Value::String(b)) => Ok(Value::String(a + &b)),
            (Value::String(a), TokenKind::Percent, b_val) => {
                // String formatting
                self.string_modulo_format(&a, &b_val, span)
            }
            _ => runtime_error(span, &format!("Unsupported binary op")),
        }
    }

    fn string_modulo_format(
        &mut self,
        fmt_str: &str,
        val: &Value,
        span: Span,
    ) -> Result<Value, EldritchError> {
        // Simple implementation of %s formatting
        let mut result = String::new();
        let mut chars = fmt_str.chars().peekable();
        let mut val_idx = 0;
        let vals: Vec<Value> = match val {
            Value::Tuple(t) => t.clone(),
            _ => vec![val.clone()],
        };

        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next) = chars.peek() {
                    if next == 's' {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        result.push_str(&vals[val_idx].to_string());
                        val_idx += 1;
                    } else if next == '%' {
                        chars.next();
                        result.push('%');
                    } else {
                        // For now only support %s and %%
                        return runtime_error(
                            span,
                            &format!("Unsupported format specifier: %{}", next),
                        );
                    }
                } else {
                    return runtime_error(span, "incomplete format");
                }
            } else {
                result.push(c);
            }
        }

        if val_idx < vals.len() {
            // It is okay if we have extra args if they were not consumed?
            // Python raises TypeError: not all arguments converted during string formatting
            return runtime_error(span, "not all arguments converted during string formatting");
        }

        Ok(Value::String(result))
    }

    fn assign(&mut self, target: &Expr, value: Value) -> Result<(), EldritchError> {
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.assign_variable(name, value);
                Ok(())
            }
            ExprKind::List(elements) | ExprKind::Tuple(elements) => {
                // Unpacking
                let values = match value {
                    Value::List(l) => l.borrow().clone(),
                    Value::Tuple(t) => t.clone(),
                    _ => {
                        return runtime_error(
                            target.span,
                            &format!("cannot unpack non-iterable {:?}", get_type_name(&value)),
                        )
                    }
                };

                if elements.len() != values.len() {
                    return runtime_error(
                        target.span,
                        &format!(
                            "ValueError: too many/not enough values to unpack (expected {}, got {})",
                            elements.len(),
                            values.len()
                        ),
                    );
                }

                for (target_elem, val_elem) in elements.iter().zip(values.into_iter()) {
                    self.assign(target_elem, val_elem)?;
                }
                Ok(())
            }
            ExprKind::Index(obj_expr, index_expr) => {
                let obj = self.evaluate(obj_expr)?;
                let index = self.evaluate(index_expr)?;
                match obj {
                    Value::List(l) => {
                        let idx_int = match index {
                            Value::Int(i) => i,
                            _ => return runtime_error(index_expr.span, "List indices must be integers"),
                        };
                        let mut list = l.borrow_mut();
                         let true_idx = if idx_int < 0 {
                            list.len() as i64 + idx_int
                        } else {
                            idx_int
                        };
                        if true_idx < 0 || true_idx as usize >= list.len() {
                            return runtime_error(target.span, "List assignment index out of range");
                        }
                        list[true_idx as usize] = value;
                        Ok(())
                    }
                    Value::Dictionary(d) => {
                         let key_str = match index {
                            Value::String(s) => s,
                            _ => return runtime_error(index_expr.span, "Dictionary keys must be strings"),
                        };
                        d.borrow_mut().insert(key_str, value);
                        Ok(())
                    }
                    _ => runtime_error(target.span, "Object does not support item assignment"),
                }
            }
            _ => runtime_error(target.span, "cannot assign to this expression"),
        }
    }

    fn execute_augmented_assignment(
        &mut self,
        target: &Expr,
        op: &TokenKind,
        value_expr: &Expr,
    ) -> Result<(), EldritchError> {
        let span = target.span;
        let right = self.evaluate(value_expr)?;

        // For simple identifiers, read, op, write
        match &target.kind {
            ExprKind::Identifier(name) => {
                let left = self.lookup_variable(name, span)?;
                let bin_op = match op {
                    TokenKind::PlusAssign => TokenKind::Plus,
                    TokenKind::MinusAssign => TokenKind::Minus,
                    TokenKind::StarAssign => TokenKind::Star,
                    TokenKind::SlashAssign => TokenKind::Slash,
                    TokenKind::SlashSlashAssign => TokenKind::SlashSlash,
                    TokenKind::PercentAssign => TokenKind::Percent,
                    _ => return runtime_error(span, "Unknown augmented assignment operator"),
                };

                // Construct dummy expressions for apply_binary_op call to reuse logic
                let left_expr = Expr { kind: ExprKind::Literal(left), span };
                let right_expr = Expr { kind: ExprKind::Literal(right), span };

                let new_val = self.apply_binary_op(&left_expr, &bin_op, &right_expr, span)?;
                self.assign_variable(name, new_val);
                Ok(())
            }
            ExprKind::Index(obj_expr, index_expr) => {
                 let obj = self.evaluate(obj_expr)?;
                 let index = self.evaluate(index_expr)?;

                 // This is tricky: we need to get the item, op it, and set it back.
                 // For mutable objects (List, Dict), we can modify in place or set item.

                 let current_val = match &obj {
                    Value::List(l) => {
                        let idx_int = match index {
                            Value::Int(i) => i,
                            _ => return runtime_error(index_expr.span, "List indices must be integers"),
                        };
                        let list = l.borrow();
                         let true_idx = if idx_int < 0 {
                            list.len() as i64 + idx_int
                        } else {
                            idx_int
                        };
                        if true_idx < 0 || true_idx as usize >= list.len() {
                            return runtime_error(span, "List index out of range");
                        }
                        list[true_idx as usize].clone()
                    }
                    Value::Dictionary(d) => {
                         let key_str = match &index {
                            Value::String(s) => s,
                            _ => return runtime_error(index_expr.span, "Dictionary keys must be strings"),
                        };
                        let dict = d.borrow();
                        match dict.get(key_str) {
                            Some(v) => v.clone(),
                            None => return runtime_error(span, "KeyError"),
                        }
                    }
                    _ => return runtime_error(span, "Object does not support item assignment"),
                 };

                let bin_op = match op {
                    TokenKind::PlusAssign => TokenKind::Plus,
                    TokenKind::MinusAssign => TokenKind::Minus,
                    TokenKind::StarAssign => TokenKind::Star,
                    TokenKind::SlashAssign => TokenKind::Slash,
                    TokenKind::SlashSlashAssign => TokenKind::SlashSlash,
                    TokenKind::PercentAssign => TokenKind::Percent,
                    _ => return runtime_error(span, "Unknown augmented assignment operator"),
                };

                 let left_expr = Expr { kind: ExprKind::Literal(current_val), span };
                 let right_expr = Expr { kind: ExprKind::Literal(right), span };
                 let new_val = self.apply_binary_op(&left_expr, &bin_op, &right_expr, span)?;

                 // Set back
                 match obj {
                    Value::List(l) => {
                        // Need to re-calculate index as borrow ends
                        let idx_int = match index {
                            Value::Int(i) => i,
                            _ => unreachable!(),
                        };
                         let mut list = l.borrow_mut();
                         let true_idx = if idx_int < 0 {
                            list.len() as i64 + idx_int
                        } else {
                            idx_int
                        };
                        list[true_idx as usize] = new_val;
                        Ok(())
                    }
                    Value::Dictionary(d) => {
                         let key_str = match index {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };
                        d.borrow_mut().insert(key_str, new_val);
                        Ok(())
                    }
                    _ => unreachable!(),
                 }

            }
            _ => runtime_error(span, "Illegal target for augmented assignment"),
        }
    }
}
