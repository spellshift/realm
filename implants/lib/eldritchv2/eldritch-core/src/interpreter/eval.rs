use super::super::ast::{
    Argument, Environment, Expr, ExprKind, FStringSegment, Function, Param, RuntimeParam, Stmt,
    StmtKind, Value,
};
use super::super::token::{Span, TokenKind};
use super::core::{Flow, Interpreter};
use super::error::{runtime_error, EldritchError};
use super::methods::call_bound_method;
use super::utils::{adjust_slice_indices, get_type_name, is_truthy};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

use super::exec::execute_stmts;

pub(crate) const MAX_RECURSION_DEPTH: usize = 64;

pub fn evaluate(interp: &mut Interpreter, expr: &Expr) -> Result<Value, EldritchError> {
    let span = expr.span;
    match &expr.kind {
        ExprKind::Literal(value) => Ok(value.clone()),
        ExprKind::Identifier(name) => interp.lookup_variable(name, span),
        ExprKind::BinaryOp(left, op, right) => apply_binary_op(interp, left, op, right, span),
        ExprKind::UnaryOp(op, right) => apply_unary_op(interp, op, right, span),
        ExprKind::LogicalOp(left, op, right) => apply_logical_op(interp, left, op, right, span),
        ExprKind::Call(callee, args) => call_function(interp, callee, args, span),
        ExprKind::List(elements) => evaluate_list_literal(interp, elements),
        ExprKind::Tuple(elements) => evaluate_tuple_literal(interp, elements),
        ExprKind::Dictionary(entries) => evaluate_dict_literal(interp, entries),
        ExprKind::Set(elements) => evaluate_set_literal(interp, elements),
        ExprKind::Index(obj, index) => evaluate_index(interp, obj, index, span),
        ExprKind::GetAttr(obj, name) => evaluate_getattr(interp, obj, name.to_string()),
        ExprKind::Slice(obj, start, stop, step) => {
            evaluate_slice(interp, obj, start, stop, step, span)
        }
        ExprKind::FString(segments) => evaluate_fstring(interp, segments),
        ExprKind::ListComp {
            body,
            var,
            iterable,
            cond,
        } => evaluate_list_comp(interp, body, var, iterable, cond),
        ExprKind::DictComp {
            key,
            value,
            var,
            iterable,
            cond,
        } => evaluate_dict_comp(interp, key, value, var, iterable, cond),
        ExprKind::SetComp {
            body,
            var,
            iterable,
            cond,
        } => evaluate_set_comp(interp, body, var, iterable, cond),
        ExprKind::Lambda { params, body } => evaluate_lambda(interp, params, body),
        ExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_val = evaluate(interp, cond)?;
            if is_truthy(&cond_val) {
                evaluate(interp, then_branch)
            } else {
                evaluate(interp, else_branch)
            }
        }
    }
}

fn evaluate_lambda(
    interp: &mut Interpreter,
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
                let val = evaluate(interp, default_expr)?;
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
        closure: interp.env.clone(),
    });
    Ok(func)
}

fn evaluate_list_comp(
    interp: &mut Interpreter,
    body: &Expr,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    let iterable_val = evaluate(interp, iterable)?;
    let items = to_iterable(interp, &iterable_val, iterable.span)?;
    let printer = interp.env.read().printer.clone();
    let comp_env = Arc::new(RwLock::new(Environment {
        parent: Some(interp.env.clone()),
        values: BTreeMap::new(),
        printer,
    }));
    let original_env = interp.env.clone();
    interp.env = comp_env;
    let mut results = Vec::new();
    for item in items {
        interp.define_variable(var, item);
        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            results.push(evaluate(interp, body)?);
        }
    }
    interp.env = original_env;
    Ok(Value::List(Arc::new(RwLock::new(results))))
}

fn evaluate_dict_comp(
    interp: &mut Interpreter,
    key_expr: &Expr,
    val_expr: &Expr,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    let iterable_val = evaluate(interp, iterable)?;
    let items = match iterable_val {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        _ => {
            return runtime_error(
                iterable.span,
                &format!("Type '{:?}' is not iterable", get_type_name(&iterable_val)),
            )
        }
    };
    let printer = interp.env.read().printer.clone();
    let comp_env = Arc::new(RwLock::new(Environment {
        parent: Some(interp.env.clone()),
        values: BTreeMap::new(),
        printer,
    }));
    let original_env = interp.env.clone();
    interp.env = comp_env;
    let mut results = BTreeMap::new();
    for item in items {
        interp.define_variable(var, item);
        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            let k = evaluate(interp, key_expr)?;
            let v = evaluate(interp, val_expr)?;
            let k_str = match k {
                Value::String(s) => s,
                _ => return runtime_error(key_expr.span, "Dict keys must be strings"),
            };
            results.insert(k_str, v);
        }
    }
    interp.env = original_env;
    Ok(Value::Dictionary(Arc::new(RwLock::new(results))))
}

fn evaluate_list_literal(
    interp: &mut Interpreter,
    elements: &[Expr],
) -> Result<Value, EldritchError> {
    let mut vals = Vec::new();
    for expr in elements {
        vals.push(evaluate(interp, expr)?);
    }
    Ok(Value::List(Arc::new(RwLock::new(vals))))
}

fn evaluate_tuple_literal(
    interp: &mut Interpreter,
    elements: &[Expr],
) -> Result<Value, EldritchError> {
    let mut vals = Vec::new();
    for expr in elements {
        vals.push(evaluate(interp, expr)?);
    }
    Ok(Value::Tuple(vals))
}

fn evaluate_dict_literal(
    interp: &mut Interpreter,
    entries: &[(Expr, Expr)],
) -> Result<Value, EldritchError> {
    let mut map = BTreeMap::new();
    for (key_expr, value_expr) in entries {
        let key_val = evaluate(interp, key_expr)?;
        let value_val = evaluate(interp, value_expr)?;
        let key_str = match key_val {
            Value::String(s) => s,
            _ => return runtime_error(key_expr.span, "Dictionary keys must be strings."),
        };
        map.insert(key_str, value_val);
    }
    Ok(Value::Dictionary(Arc::new(RwLock::new(map))))
}

fn evaluate_set_comp(
    interp: &mut Interpreter,
    body: &Expr,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    let iterable_val = evaluate(interp, iterable)?;
    let items = match iterable_val {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        _ => {
            return runtime_error(
                iterable.span,
                &format!("Type '{:?}' is not iterable", get_type_name(&iterable_val)),
            )
        }
    };
    let printer = interp.env.read().printer.clone();
    let comp_env = Arc::new(RwLock::new(Environment {
        parent: Some(interp.env.clone()),
        values: BTreeMap::new(),
        printer,
    }));
    let original_env = interp.env.clone();
    interp.env = comp_env;
    #[allow(clippy::mutable_key_type)]
    let mut results = BTreeSet::new();
    for item in items {
        interp.define_variable(var, item);
        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            results.insert(evaluate(interp, body)?);
        }
    }
    interp.env = original_env;
    Ok(Value::Set(Arc::new(RwLock::new(results))))
}

fn evaluate_set_literal(
    interp: &mut Interpreter,
    elements: &[Expr],
) -> Result<Value, EldritchError> {
    #[allow(clippy::mutable_key_type)]
    let mut set = BTreeSet::new();
    for expr in elements {
        let val = evaluate(interp, expr)?;
        set.insert(val);
    }
    Ok(Value::Set(Arc::new(RwLock::new(set))))
}

fn evaluate_index(
    interp: &mut Interpreter,
    obj: &Expr,
    index: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;
    let idx_val = evaluate(interp, index)?;

    match obj_val {
        Value::List(l) => {
            let idx_int = match idx_val {
                Value::Int(i) => i,
                _ => return runtime_error(index.span, "List indices must be integers"),
            };
            let list = l.read();
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
            let dict = d.read();
            match dict.get(&key_str) {
                Some(v) => Ok(v.clone()),
                None => runtime_error(span, &format!("KeyError: '{key_str}'")),
            }
        }
        _ => runtime_error(obj.span, &format!("Type not subscriptable: {obj_val:?}")),
    }
}

fn evaluate_slice(
    interp: &mut Interpreter,
    obj: &Expr,
    start: &Option<Box<Expr>>,
    stop: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
    span: Span,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;

    let step_val = if let Some(s) = step {
        match evaluate(interp, s)? {
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
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => return runtime_error(s.span, "Slice start must be integer"),
        }
    } else {
        None
    };

    let stop_val_opt = if let Some(s) = stop {
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => return runtime_error(s.span, "Slice stop must be integer"),
        }
    } else {
        None
    };

    match obj_val {
        Value::List(l) => {
            let list = l.read();
            let len = list.len() as i64;
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);

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
            Ok(Value::List(Arc::new(RwLock::new(result))))
        }
        Value::Tuple(t) => {
            let len = t.len() as i64;
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
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
            let (i, j) = adjust_slice_indices(len, &start_val_opt, &stop_val_opt, step_val);
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
        _ => runtime_error(obj.span, &format!("Type not sliceable: {obj_val:?}")),
    }
}

fn evaluate_getattr(
    interp: &mut Interpreter,
    obj: &Expr,
    name: String,
) -> Result<Value, EldritchError> {
    let obj_val = evaluate(interp, obj)?;

    // Support dot access for dictionary keys (useful for modules)
    if let Value::Dictionary(d) = &obj_val {
        if let Some(val) = d.read().get(&name) {
            return Ok(val.clone());
        }
    }

    // Support Foreign Objects
    if let Value::Foreign(_) = &obj_val {
        // Return a bound method where the receiver is the foreign object
        return Ok(Value::BoundMethod(Box::new(obj_val), name));
    }

    Ok(Value::BoundMethod(Box::new(obj_val), name))
}

fn evaluate_fstring(
    interp: &mut Interpreter,
    segments: &[FStringSegment],
) -> Result<Value, EldritchError> {
    let mut parts = Vec::new();
    for segment in segments {
        match segment {
            FStringSegment::Literal(s) => parts.push(s.clone()),
            FStringSegment::Expression(expr) => {
                let val = evaluate(interp, expr)?;
                parts.push(val.to_string());
            }
        }
    }
    Ok(Value::String(parts.join("")))
}

fn call_function(
    interp: &mut Interpreter,
    callee: &Expr,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    let callee_val = evaluate(interp, callee)?;

    // Special handling for map/filter/reduce which take functions
    if let Value::NativeFunction(name, _) = &callee_val {
        if name == "map" {
            return builtin_map(interp, args, span);
        } else if name == "filter" {
            return builtin_filter(interp, args, span);
        } else if name == "reduce" {
            return builtin_reduce(interp, args, span);
        }
    }

    // Standard call
    let mut pos_args_val = Vec::new();
    let mut kw_args_val = BTreeMap::new();

    for arg in args {
        match arg {
            Argument::Positional(expr) => pos_args_val.push(evaluate(interp, expr)?),
            Argument::Keyword(name, expr) => {
                let val = evaluate(interp, expr)?;
                kw_args_val.insert(name.clone(), val);
            }
            Argument::StarArgs(expr) => {
                let val = evaluate(interp, expr)?;
                match val {
                    Value::List(l) => pos_args_val.extend(l.read().clone()),
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
                let val = evaluate(interp, expr)?;
                match val {
                    Value::Dictionary(d) => kw_args_val.extend(d.read().clone()),
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
            if !kw_args_val.is_empty() {
                return runtime_error(span, "NativeFunction does not accept keyword arguments");
            }
            f(&interp.env, args_slice).map_err(|e| EldritchError { message: e, span })
        }
        Value::NativeFunctionWithKwargs(_, f) => {
            f(&interp.env, args_slice, &kw_args_val).map_err(|e| EldritchError { message: e, span })
        }
        Value::Function(Function {
            name,
            params,
            body,
            closure,
        }) => {
            #[allow(unused_variables)]
            let _ = name; // Silence unused name warning if any

            if interp.depth >= MAX_RECURSION_DEPTH {
                return runtime_error(span, "Recursion limit exceeded");
            }
            interp.depth += 1;

            let result = (|| {
                let printer = interp.env.read().printer.clone();
                let function_env = Arc::new(RwLock::new(Environment {
                    parent: Some(closure),
                    values: BTreeMap::new(),
                    printer,
                }));
                let mut pos_idx = 0;
                for param in params {
                    match param {
                        RuntimeParam::Normal(param_name) => {
                            if pos_idx < pos_args_val.len() {
                                function_env
                                    .write()
                                    .values
                                    .insert(param_name.clone(), pos_args_val[pos_idx].clone());
                                pos_idx += 1;
                            } else if let Some(val) = kw_args_val.remove(&param_name) {
                                function_env
                                    .write()
                                    .values
                                    .insert(param_name.clone(), val);
                            } else {
                                return runtime_error(
                                    span,
                                    &format!("Missing required argument: '{param_name}'"),
                                );
                            }
                        }
                        RuntimeParam::WithDefault(param_name, default_val) => {
                            if pos_idx < pos_args_val.len() {
                                function_env
                                    .write()
                                    .values
                                    .insert(param_name.clone(), pos_args_val[pos_idx].clone());
                                pos_idx += 1;
                            } else if let Some(val) = kw_args_val.remove(&param_name) {
                                function_env
                                    .write()
                                    .values
                                    .insert(param_name.clone(), val);
                            } else {
                                function_env
                                    .write()
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
                                .write()
                                .values
                                .insert(param_name.clone(), Value::Tuple(remaining));
                        }
                        RuntimeParam::StarStar(param_name) => {
                            let mut dict = BTreeMap::new();
                            let keys_to_move: Vec<String> = kw_args_val.keys().cloned().collect();
                            for k in keys_to_move {
                                if let Some(v) = kw_args_val.remove(&k) {
                                    dict.insert(k, v);
                                }
                            }
                            function_env.write().values.insert(
                                param_name.clone(),
                                Value::Dictionary(Arc::new(RwLock::new(dict))),
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
                        &format!("Function '{name}' got unexpected keyword arguments: {keys:?}"),
                    );
                }

                let original_env = interp.env.clone();
                interp.env = function_env;
                let old_flow = interp.flow.clone();
                interp.flow = Flow::Next;

                execute_stmts(interp, &body)?;

                let ret_val = if let Flow::Return(v) = &interp.flow {
                    v.clone()
                } else {
                    Value::None
                };
                interp.env = original_env;
                interp.flow = old_flow;
                Ok(ret_val)
            })();
            interp.depth -= 1;
            result
        }
        Value::BoundMethod(receiver, method_name) => {
            // Check if receiver is Foreign
            if let Value::Foreign(foreign) = receiver.as_ref() {
                foreign
                    .call_method(&method_name, args_slice, &kw_args_val)
                    .map_err(|e| EldritchError { message: e, span })
            } else {
                if !kw_args_val.is_empty() {
                    return runtime_error(span, "BoundMethod does not accept keyword arguments");
                }
                call_bound_method(&receiver, &method_name, args_slice)
                    .map_err(|e| EldritchError { message: e, span })
            }
        }
        _ => runtime_error(span, &format!("Cannot call value of type: {callee_val:?}")),
    }
}

fn builtin_map(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 2 {
        return runtime_error(span, "map() takes exactly 2 arguments");
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;

    let items = to_iterable(interp, &iterable_val, span)?;
    let mut results = Vec::new();

    for item in items {
        let res = call_value(interp, &func_val, core::slice::from_ref(&item), span)?;
        results.push(res);
    }

    Ok(Value::List(Arc::new(RwLock::new(results))))
}

fn builtin_filter(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 2 {
        return runtime_error(span, "filter() takes exactly 2 arguments");
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let items = to_iterable(interp, &iterable_val, span)?;

    let mut results = Vec::new();
    for item in items {
        let keep = if let Value::None = func_val {
            is_truthy(&item)
        } else {
            let res = call_value(interp, &func_val, core::slice::from_ref(&item), span)?;
            is_truthy(&res)
        };
        if keep {
            results.push(item);
        }
    }
    Ok(Value::List(Arc::new(RwLock::new(results))))
}

fn builtin_reduce(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() < 2 || args.len() > 3 {
        return runtime_error(span, "reduce() takes 2 or 3 arguments");
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let mut items = to_iterable(interp, &iterable_val, span)?.into_iter();

    let mut acc = if args.len() == 3 {
        evaluate_arg(interp, &args[2])?
    } else {
        match items.next() {
            Some(v) => v,
            None => return runtime_error(span, "reduce() of empty sequence with no initial value"),
        }
    };

    for item in items {
        acc = call_value(interp, &func_val, &[acc, item], span)?;
    }
    Ok(acc)
}

fn call_value(
    interp: &mut Interpreter,
    func: &Value,
    args: &[Value],
    span: Span,
) -> Result<Value, EldritchError> {
    match func {
        Value::NativeFunction(_, f) => {
            f(&interp.env, args).map_err(|e| EldritchError { message: e, span })
        }
        Value::Function(Function {
            name: _,
            params: _,
            body: _,
            closure: _,
        }) => {
            if interp.depth >= MAX_RECURSION_DEPTH {
                return runtime_error(span, "Recursion limit exceeded");
            }
            interp.depth += 1;

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

            let res = call_function(interp, &callee_expr, &expr_args, span);
            interp.depth -= 1;
            res
        }
        Value::BoundMethod(receiver, method_name) => call_bound_method(receiver, method_name, args)
            .map_err(|e| EldritchError { message: e, span }),
        _ => runtime_error(span, "not callable"),
    }
}

fn evaluate_arg(interp: &mut Interpreter, arg: &Argument) -> Result<Value, EldritchError> {
    match arg {
        Argument::Positional(e) => evaluate(interp, e),
        // Just return a dummy span here for the error, or match e.span if available.
        // Since we don't have easy access to a span here without unpacking, use a dummy one.
        _ => runtime_error(
            Span::new(0, 0, 0),
            "HOFs currently only support positional arguments",
        ),
    }
}

fn to_iterable(
    _interp: &Interpreter,
    val: &Value,
    span: Span,
) -> Result<Vec<Value>, EldritchError> {
    match val {
        Value::List(l) => Ok(l.read().clone()),
        Value::Tuple(t) => Ok(t.clone()),
        Value::Set(s) => Ok(s.read().iter().cloned().collect()),
        Value::Dictionary(d) => Ok(d
            .read()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        _ => runtime_error(
            span,
            &format!("Type '{:?}' is not iterable", get_type_name(val)),
        ),
    }
}

fn apply_unary_op(
    interp: &mut Interpreter,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let val = evaluate(interp, right)?;
    match op {
        TokenKind::Minus => match val {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => runtime_error(span, "Unary '-' only valid for numbers"),
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
    interp: &mut Interpreter,
    left: &Expr,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let left_val = evaluate(interp, left)?;
    match op {
        TokenKind::Or => {
            if is_truthy(&left_val) {
                return Ok(left_val);
            }
            evaluate(interp, right)
        }
        TokenKind::And => {
            if !is_truthy(&left_val) {
                return Ok(left_val);
            }
            evaluate(interp, right)
        }
        _ => runtime_error(span, "Invalid logical operator"),
    }
}

fn evaluate_in(
    _interp: &mut Interpreter,
    item: &Value,
    collection: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    match collection {
        Value::List(l) => {
            let list = l.read();
            Ok(Value::Bool(list.contains(item)))
        }
        Value::Tuple(t) => Ok(Value::Bool(t.contains(item))),
        Value::Dictionary(d) => {
            let dict = d.read();
            // Check keys
            let key = match item {
                Value::String(s) => s,
                _ => return Ok(Value::Bool(false)), // Only strings are keys
            };
            Ok(Value::Bool(dict.contains_key(key)))
        }
        Value::Set(s) => {
            let set = s.read();
            Ok(Value::Bool(set.contains(item)))
        }
        Value::String(s) => {
            let sub = match item {
                Value::String(ss) => ss,
                _ => return runtime_error(span, "'in <string>' requires string as left operand"),
            };
            Ok(Value::Bool(s.contains(sub)))
        }
        _ => runtime_error(
            span,
            &format!(
                "argument of type '{}' is not iterable",
                get_type_name(collection)
            ),
        ),
    }
}

fn apply_binary_op(
    interp: &mut Interpreter,
    left: &Expr,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    let a = evaluate(interp, left)?;
    let b = evaluate(interp, right)?;

    match (a, op.clone(), b) {
        // Mixed arithmetic equality
        (Value::Int(a), TokenKind::Eq, Value::Float(b)) => Ok(Value::Bool(a as f64 == b)),
        (Value::Float(a), TokenKind::Eq, Value::Int(b)) => Ok(Value::Bool(a == b as f64)),
        (Value::Int(a), TokenKind::NotEq, Value::Float(b)) => Ok(Value::Bool(a as f64 != b)),
        (Value::Float(a), TokenKind::NotEq, Value::Int(b)) => Ok(Value::Bool(a != b as f64)),

        (a, TokenKind::Eq, b) => Ok(Value::Bool(a == b)),
        (a, TokenKind::NotEq, b) => Ok(Value::Bool(a != b)),

        // INT Comparisons
        (Value::Int(a), TokenKind::Lt, Value::Int(b)) => Ok(Value::Bool(a < b)),
        (Value::Int(a), TokenKind::Gt, Value::Int(b)) => Ok(Value::Bool(a > b)),
        (Value::Int(a), TokenKind::LtEq, Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (Value::Int(a), TokenKind::GtEq, Value::Int(b)) => Ok(Value::Bool(a >= b)),

        // FLOAT Comparisons
        (Value::Float(a), TokenKind::Lt, Value::Float(b)) => Ok(Value::Bool(a < b)),
        (Value::Float(a), TokenKind::Gt, Value::Float(b)) => Ok(Value::Bool(a > b)),
        (Value::Float(a), TokenKind::LtEq, Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (Value::Float(a), TokenKind::GtEq, Value::Float(b)) => Ok(Value::Bool(a >= b)),

        // MIXED Comparisons (Int vs Float)
        (Value::Int(a), TokenKind::Lt, Value::Float(b)) => Ok(Value::Bool((a as f64) < b)),
        (Value::Int(a), TokenKind::Gt, Value::Float(b)) => Ok(Value::Bool((a as f64) > b)),
        (Value::Int(a), TokenKind::LtEq, Value::Float(b)) => Ok(Value::Bool((a as f64) <= b)),
        (Value::Int(a), TokenKind::GtEq, Value::Float(b)) => Ok(Value::Bool((a as f64) >= b)),

        (Value::Float(a), TokenKind::Lt, Value::Int(b)) => Ok(Value::Bool(a < (b as f64))),
        (Value::Float(a), TokenKind::Gt, Value::Int(b)) => Ok(Value::Bool(a > (b as f64))),
        (Value::Float(a), TokenKind::LtEq, Value::Int(b)) => Ok(Value::Bool(a <= (b as f64))),
        (Value::Float(a), TokenKind::GtEq, Value::Int(b)) => Ok(Value::Bool(a >= (b as f64))),

        // STRING Comparisons
        (Value::String(a), TokenKind::Lt, Value::String(b)) => Ok(Value::Bool(a < b)),
        (Value::String(a), TokenKind::Gt, Value::String(b)) => Ok(Value::Bool(a > b)),
        (Value::String(a), TokenKind::LtEq, Value::String(b)) => Ok(Value::Bool(a <= b)),
        (Value::String(a), TokenKind::GtEq, Value::String(b)) => Ok(Value::Bool(a >= b)),

        // IN Operator
        (item, TokenKind::In, collection) => evaluate_in(interp, &item, &collection, span),
        (item, TokenKind::NotIn, collection) => {
            let res = evaluate_in(interp, &item, &collection, span)?;
            match res {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => unreachable!("evaluate_in always returns boolean or error"),
            }
        }

        // Arithmetic
        (Value::Int(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Int(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Int(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Int(a), TokenKind::Slash, Value::Int(b)) => {
            if b == 0 {
                return runtime_error(span, "divide by zero");
            }
            // User requirement: / for integers returns float
            Ok(Value::Float((a as f64) / (b as f64)))
        }

        // Float Arithmetic
        (Value::Float(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(a), TokenKind::Slash, Value::Float(b)) => Ok(Value::Float(a / b)),
        (Value::Float(a), TokenKind::SlashSlash, Value::Float(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / b)))
            }
        } // Floor div for float
        // Mixed Arithmetic
        (Value::Int(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float((a as f64) + b)),
        (Value::Int(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float((a as f64) - b)),
        (Value::Int(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float((a as f64) * b)),
        (Value::Int(a), TokenKind::Slash, Value::Float(b)) => Ok(Value::Float((a as f64) / b)),

        (Value::Float(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Float(a + (b as f64))),
        (Value::Float(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Float(a - (b as f64))),
        (Value::Float(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Float(a * (b as f64))),
        (Value::Float(a), TokenKind::Slash, Value::Int(b)) => Ok(Value::Float(a / (b as f64))),

        (Value::Int(a), TokenKind::SlashSlash, Value::Float(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((a as f64).div_euclid(b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a as f64 / b)))
            }
        }
        (Value::Float(a), TokenKind::SlashSlash, Value::Int(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / b as f64)))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Float(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Int(a), TokenKind::Percent, Value::Float(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((a as f64).rem_euclid(b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let a = a as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Int(b)) => {
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                let b = b as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
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

        // Set Bitwise operations
        (Value::Set(a), TokenKind::BitAnd, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let intersection: BTreeSet<Value> =
                a.read().intersection(&b.read()).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(intersection))))
        }
        (Value::Set(a), TokenKind::BitOr, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let union: BTreeSet<Value> = a.read().union(&b.read()).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(union))))
        }
        (Value::Set(a), TokenKind::BitXor, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let symmetric_difference: BTreeSet<Value> = a
                .read()
                .symmetric_difference(&b.read())
                .cloned()
                .collect();
            Ok(Value::Set(Arc::new(RwLock::new(symmetric_difference))))
        }
        (Value::Set(a), TokenKind::Minus, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let difference: BTreeSet<Value> = a.read().difference(&b.read()).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(difference))))
        }

        (Value::String(a), TokenKind::Plus, Value::String(b)) => Ok(Value::String(a + &b)),
        (Value::String(a), TokenKind::Percent, b_val) => {
            // String formatting
            string_modulo_format(interp, &a, &b_val, span)
        }

        (Value::Bytes(a), TokenKind::Plus, Value::Bytes(b)) => {
            let mut new_bytes = a.clone();
            new_bytes.extend(b.iter());
            Ok(Value::Bytes(new_bytes))
        }

        (Value::Bytes(a), TokenKind::Star, Value::Int(n)) => {
            if n <= 0 {
                Ok(Value::Bytes(Vec::new()))
            } else {
                let mut new_bytes = Vec::with_capacity(a.len() * (n as usize));
                for _ in 0..n {
                    new_bytes.extend(a.iter());
                }
                Ok(Value::Bytes(new_bytes))
            }
        }
        (Value::Int(n), TokenKind::Star, Value::Bytes(a)) => {
            if n <= 0 {
                Ok(Value::Bytes(Vec::new()))
            } else {
                let mut new_bytes = Vec::with_capacity(a.len() * (n as usize));
                for _ in 0..n {
                    new_bytes.extend(a.iter());
                }
                Ok(Value::Bytes(new_bytes))
            }
        }

        // List concatenation (new list)
        (Value::List(a), TokenKind::Plus, Value::List(b)) => {
            let mut new_list = a.read().clone();
            new_list.extend(b.read().clone());
            Ok(Value::List(Arc::new(RwLock::new(new_list))))
        }

        // List repetition (Multiplication)
        (Value::List(a), TokenKind::Star, Value::Int(n)) => {
            let mut new_list = Vec::new();
            if n > 0 {
                let list_ref = a.read();
                for _ in 0..n {
                    new_list.extend(list_ref.clone());
                }
            }
            Ok(Value::List(Arc::new(RwLock::new(new_list))))
        }
        (Value::Int(n), TokenKind::Star, Value::List(a)) => {
            let mut new_list = Vec::new();
            if n > 0 {
                let list_ref = a.read();
                for _ in 0..n {
                    new_list.extend(list_ref.clone());
                }
            }
            Ok(Value::List(Arc::new(RwLock::new(new_list))))
        }

        // Tuple concatenation (new tuple)
        (Value::Tuple(a), TokenKind::Plus, Value::Tuple(b)) => {
            let mut new_tuple = a.clone();
            new_tuple.extend(b.clone());
            Ok(Value::Tuple(new_tuple))
        }

        // Tuple repetition
        (Value::Tuple(a), TokenKind::Star, Value::Int(n)) => {
            let mut new_tuple = Vec::new();
            if n > 0 {
                for _ in 0..n {
                    new_tuple.extend(a.clone());
                }
            }
            Ok(Value::Tuple(new_tuple))
        }
        (Value::Int(n), TokenKind::Star, Value::Tuple(a)) => {
            let mut new_tuple = Vec::new();
            if n > 0 {
                for _ in 0..n {
                    new_tuple.extend(a.clone());
                }
            }
            Ok(Value::Tuple(new_tuple))
        }

        // String repetition
        (Value::String(s), TokenKind::Star, Value::Int(n)) => {
            if n <= 0 {
                Ok(Value::String(String::new()))
            } else {
                Ok(Value::String(s.repeat(n as usize)))
            }
        }
        (Value::Int(n), TokenKind::Star, Value::String(s)) => {
            if n <= 0 {
                Ok(Value::String(String::new()))
            } else {
                Ok(Value::String(s.repeat(n as usize)))
            }
        }

        // Sequence Comparisons
        (Value::List(a), op, Value::List(b)) => {
            let list_a = a.read();
            let list_b = b.read();
            compare_sequences(&list_a, &list_b, op, span)
        }
        (Value::Tuple(a), op, Value::Tuple(b)) => compare_sequences(&a, &b, op, span),

        // Dict merge (new dict)
        (Value::Dictionary(a), TokenKind::Plus, Value::Dictionary(b))
        | (Value::Dictionary(a), TokenKind::BitOr, Value::Dictionary(b)) => {
            let mut new_dict = a.read().clone();
            for (k, v) in b.read().iter() {
                new_dict.insert(k.clone(), v.clone());
            }
            Ok(Value::Dictionary(Arc::new(RwLock::new(new_dict))))
        }

        // Set union (new set) - Plus is deprecated for sets in favor of |
        (Value::Set(a), TokenKind::Plus, Value::Set(b)) => {
            #[allow(clippy::mutable_key_type)]
            let mut new_set = a.read().clone();
            for item in b.read().iter() {
                new_set.insert(item.clone());
            }
            Ok(Value::Set(Arc::new(RwLock::new(new_set))))
        }

        _ => runtime_error(span, "Unsupported binary op"),
    }
}

fn string_modulo_format(
    _interp: &mut Interpreter,
    fmt_str: &str,
    val: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
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
                match next {
                    's' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        result.push_str(&vals[val_idx].to_string());
                        val_idx += 1;
                    }
                    'd' | 'i' | 'u' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let i_val = match v {
                            Value::Int(i) => *i,
                            Value::Float(f) => *f as i64,
                            Value::Bool(b) => {
                                if *b {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%{} format: a number is required, not {:?}",
                                        next,
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{}", i_val));
                        val_idx += 1;
                    }
                    'o' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let i_val = match v {
                            Value::Int(i) => *i,
                            Value::Float(f) => *f as i64,
                            Value::Bool(b) => {
                                if *b {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%o format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:o}", i_val));
                        val_idx += 1;
                    }
                    'x' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let i_val = match v {
                            Value::Int(i) => *i,
                            Value::Float(f) => *f as i64,
                            Value::Bool(b) => {
                                if *b {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%x format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:x}", i_val));
                        val_idx += 1;
                    }
                    'X' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let i_val = match v {
                            Value::Int(i) => *i,
                            Value::Float(f) => *f as i64,
                            Value::Bool(b) => {
                                if *b {
                                    1
                                } else {
                                    0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%X format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:X}", i_val));
                        val_idx += 1;
                    }
                    'e' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let f_val = match v {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            Value::Bool(b) => {
                                if *b {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%e format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:e}", f_val));
                        val_idx += 1;
                    }
                    'E' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let f_val = match v {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            Value::Bool(b) => {
                                if *b {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%E format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:E}", f_val));
                        val_idx += 1;
                    }
                    'f' | 'F' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let f_val = match v {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            Value::Bool(b) => {
                                if *b {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%f format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        // Python defaults to 6 decimal places for %f
                        result.push_str(&format!("{:.6}", f_val));
                        val_idx += 1;
                    }
                    'g' | 'G' => {
                        // Rust doesn't have direct %g support in std::fmt (general format)
                        // It uses Debug or Display which are usually compact.
                        // However, %g implies trying to keep it short.
                        // {:?} (Debug) for f64 often does a good job.
                        // Or we can just use Display {} which is usually what we want.
                        // Python's %g logic is complex. We will approximate with {:?} to ensure floats look like floats
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        let v = &vals[val_idx];
                        let f_val = match v {
                            Value::Int(i) => *i as f64,
                            Value::Float(f) => *f,
                            Value::Bool(b) => {
                                if *b {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            _ => {
                                return runtime_error(
                                    span,
                                    &format!(
                                        "%g format: a number is required, not {:?}",
                                        get_type_name(v)
                                    ),
                                )
                            }
                        };
                        result.push_str(&format!("{:?}", f_val));
                        val_idx += 1;
                    }
                    'r' => {
                        chars.next();
                        if val_idx >= vals.len() {
                            return runtime_error(span, "not enough arguments for format string");
                        }
                        match &vals[val_idx] {
                            Value::String(s) => result.push_str(&format!("\"{}\"", s)), // Simple repr
                            v => result.push_str(&v.to_string()),
                        }
                        val_idx += 1;
                    }
                    '%' => {
                        chars.next();
                        result.push('%');
                    }
                    _ => {
                        return runtime_error(span, &format!("Unsupported format specifier: %{next}"));
                    }
                }
            } else {
                return runtime_error(span, "incomplete format");
            }
        } else {
            result.push(c);
        }
    }

    if val_idx < vals.len() {
        // Python raises TypeError
        return runtime_error(span, "not all arguments converted during string formatting");
    }

    Ok(Value::String(result))
}

fn compare_sequences(
    seq_a: &[Value],
    seq_b: &[Value],
    op: TokenKind,
    span: Span,
) -> Result<Value, EldritchError> {
    // Lexicographical comparison
    let len_a = seq_a.len();
    let len_b = seq_b.len();
    let len = len_a.min(len_b);

    for i in 0..len {
        let val_a = &seq_a[i];
        let val_b = &seq_b[i];

        if val_a != val_b {
            // Need recursive comparison logic or reuse binary op?
            // Reusing apply_binary_op requires mutable interp which we don't have easily here in this helper unless passed.
            // But we have values. Let's do simple comparison if types match and are orderable.
            // For full correctness, we should recurse.
            // BUT, `apply_binary_op` takes `&Expr`. We have `Value`.
            // We need `compare_values` helper.
            return match op {
                TokenKind::Eq => Ok(Value::Bool(false)),
                TokenKind::NotEq => Ok(Value::Bool(true)),
                TokenKind::Lt => {
                    // Check if a < b
                    Ok(Value::Bool(super::utils::compare_values(val_a, val_b).map_or(
                        false,
                        |ord| matches!(ord, core::cmp::Ordering::Less),
                    )))
                }
                TokenKind::Gt => Ok(Value::Bool(super::utils::compare_values(val_a, val_b).map_or(
                    false,
                    |ord| matches!(ord, core::cmp::Ordering::Greater),
                ))),
                TokenKind::LtEq => Ok(Value::Bool(super::utils::compare_values(val_a, val_b)
                    .map_or(false, |ord| matches!(
                        ord,
                        core::cmp::Ordering::Less | core::cmp::Ordering::Equal
                    )))),
                TokenKind::GtEq => Ok(Value::Bool(super::utils::compare_values(val_a, val_b)
                    .map_or(false, |ord| matches!(
                        ord,
                        core::cmp::Ordering::Greater | core::cmp::Ordering::Equal
                    )))),
                _ => runtime_error(span, "Invalid comparison operator for sequences"),
            };
        }
    }

    // If prefix matches, compare lengths
    match op {
        TokenKind::Eq => Ok(Value::Bool(len_a == len_b)),
        TokenKind::NotEq => Ok(Value::Bool(len_a != len_b)),
        TokenKind::Lt => Ok(Value::Bool(len_a < len_b)),
        TokenKind::Gt => Ok(Value::Bool(len_a > len_b)),
        TokenKind::LtEq => Ok(Value::Bool(len_a <= len_b)),
        TokenKind::GtEq => Ok(Value::Bool(len_a >= len_b)),
        _ => runtime_error(span, "Invalid comparison operator for sequences"),
    }
}

pub(crate) fn apply_binary_op_pub(
    interp: &mut Interpreter,
    left: &Expr,
    op: &TokenKind,
    right: &Expr,
    span: Span,
) -> Result<Value, EldritchError> {
    apply_binary_op(interp, left, op, right, span)
}
