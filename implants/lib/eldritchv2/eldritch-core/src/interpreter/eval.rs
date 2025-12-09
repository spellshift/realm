use super::super::ast::{
    Argument, Environment, Expr, ExprKind, FStringSegment, Function, Param, RuntimeParam, Stmt,
    StmtKind, Value,
};
use super::super::token::{Span, TokenKind};
use super::core::{Flow, Interpreter};
use super::error::{runtime_error, EldritchError, EldritchErrorKind};
use super::introspection::{get_type_name, is_truthy};
use super::methods::call_bound_method;
use super::operations::{
    adjust_slice_indices, apply_arithmetic_op, apply_bitwise_op, apply_comparison_op,
    evaluate_comprehension_generic,
};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;
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
            Param::Normal(n, _type) => runtime_params.push(RuntimeParam::Normal(n.clone())),
            Param::Star(n, _type) => runtime_params.push(RuntimeParam::Star(n.clone())),
            Param::StarStar(n, _type) => runtime_params.push(RuntimeParam::StarStar(n.clone())),
            Param::WithDefault(n, _type, default_expr) => {
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
    let mut results = Vec::new();
    evaluate_comprehension_generic(interp, var, iterable, cond, |i| {
        results.push(evaluate(i, body)?);
        Ok(())
    })?;
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
    let mut results = BTreeMap::new();
    evaluate_comprehension_generic(interp, var, iterable, cond, |i| {
        let k = evaluate(i, key_expr)?;
        let v = evaluate(i, val_expr)?;
        results.insert(k, v);
        Ok(())
    })?;
    Ok(Value::Dictionary(Arc::new(RwLock::new(results))))
}

fn evaluate_set_comp(
    interp: &mut Interpreter,
    body: &Expr,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    #[allow(clippy::mutable_key_type)]
    let mut results = BTreeSet::new();
    evaluate_comprehension_generic(interp, var, iterable, cond, |i| {
        results.insert(evaluate(i, body)?);
        Ok(())
    })?;
    Ok(Value::Set(Arc::new(RwLock::new(results))))
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
        map.insert(key_val, value_val);
    }
    Ok(Value::Dictionary(Arc::new(RwLock::new(map))))
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
                _ => return interp.error(EldritchErrorKind::TypeError, "List indices must be integers", index.span),
            };
            let list = l.read();
            let true_idx = if idx_int < 0 {
                list.len() as i64 + idx_int
            } else {
                idx_int
            };
            if true_idx < 0 || true_idx as usize >= list.len() {
                return interp.error(EldritchErrorKind::IndexError, "List index out of range", span);
            }
            Ok(list[true_idx as usize].clone())
        }
        Value::Tuple(t) => {
            let idx_int = match idx_val {
                Value::Int(i) => i,
                _ => return interp.error(EldritchErrorKind::TypeError, "Tuple indices must be integers", index.span),
            };
            let true_idx = if idx_int < 0 {
                t.len() as i64 + idx_int
            } else {
                idx_int
            };
            if true_idx < 0 || true_idx as usize >= t.len() {
                return interp.error(EldritchErrorKind::IndexError, "Tuple index out of range", span);
            }
            Ok(t[true_idx as usize].clone())
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            match dict.get(&idx_val) {
                Some(v) => Ok(v.clone()),
                None => interp.error(EldritchErrorKind::KeyError, &format!("KeyError: '{idx_val}'"), span),
            }
        }
        _ => interp.error(EldritchErrorKind::TypeError, &format!("Type not subscriptable: {obj_val:?}"), obj.span),
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
            _ => return interp.error(EldritchErrorKind::TypeError, "Slice step must be integer", s.span),
        }
    } else {
        1
    };

    if step_val == 0 {
        return interp.error(EldritchErrorKind::ValueError, "slice step cannot be zero", span);
    }

    let start_val_opt = if let Some(s) = start {
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => return interp.error(EldritchErrorKind::TypeError, "Slice start must be integer", s.span),
        }
    } else {
        None
    };

    let stop_val_opt = if let Some(s) = stop {
        match evaluate(interp, s)? {
            Value::Int(i) => Some(i),
            _ => return interp.error(EldritchErrorKind::TypeError, "Slice stop must be integer", s.span),
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
        _ => interp.error(EldritchErrorKind::TypeError, &format!("Type not sliceable: {obj_val:?}"), obj.span),
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
        if let Some(val) = d.read().get(&Value::String(name.clone())) {
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
        } else if name == "sorted" {
            return builtin_sorted(interp, args, span);
        } else if name == "eval" {
            return builtin_eval(interp, args, span);
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
                        return interp.error(
                            EldritchErrorKind::TypeError,
                            &format!(
                                "*args argument must be iterable, got {:?}",
                                get_type_name(&val)
                            ),
                            expr.span,
                        )
                    }
                }
            }
            Argument::KwArgs(expr) => {
                let val = evaluate(interp, expr)?;
                match val {
                    Value::Dictionary(d) => {
                        let dict = d.read();
                        for (k, v) in dict.iter() {
                            match k {
                                Value::String(s) => {
                                    kw_args_val.insert(s.clone(), v.clone());
                                }
                                _ => return interp.error(EldritchErrorKind::TypeError, "Keywords must be strings", expr.span),
                            }
                        }
                    }
                    _ => {
                        return interp.error(
                            EldritchErrorKind::TypeError,
                            &format!(
                                "**kwargs argument must be a dict, got {:?}",
                                get_type_name(&val)
                            ),
                            expr.span,
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
                return interp.error(EldritchErrorKind::TypeError, "NativeFunction does not accept keyword arguments", span);
            }
            // Ensure stack frame for native call
            // Native function name?
            if let ExprKind::Identifier(name) = &callee.kind {
                 interp.push_frame(name, span);
            } else {
                 interp.push_frame("<native>", span);
            }

            let res = f(&interp.env, args_slice).map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()));
            interp.pop_frame();
            res
        }
        Value::NativeFunctionWithKwargs(_, f) => {
             // Ensure stack frame for native call
            if let ExprKind::Identifier(name) = &callee.kind {
                 interp.push_frame(name, span);
            } else {
                 interp.push_frame("<native>", span);
            }
            let res = f(&interp.env, args_slice, &kw_args_val).map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()));
            interp.pop_frame();
            res
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
                return interp.error(EldritchErrorKind::RecursionError, "Recursion limit exceeded", span);
            }
            interp.depth += 1;

            // Push stack frame
            interp.push_frame(&name, span);

            let result = (|| {
                let printer = interp.env.read().printer.clone();
                let function_env = Arc::new(RwLock::new(Environment {
                    parent: Some(closure),
                    values: BTreeMap::new(),
                    printer,
                    libraries: BTreeSet::new(),
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
                                function_env.write().values.insert(param_name.clone(), val);
                            } else {
                                return interp.error(
                                    EldritchErrorKind::TypeError,
                                    &format!("Missing required argument: '{param_name}'"),
                                    span,
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
                                function_env.write().values.insert(param_name.clone(), val);
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
                                    dict.insert(Value::String(k), v);
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
                    return interp.error(EldritchErrorKind::TypeError, "Function got too many positional arguments.", span);
                }

                if !kw_args_val.is_empty() {
                    let mut keys: Vec<&String> = kw_args_val.keys().collect();
                    keys.sort();
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &format!("Function '{name}' got unexpected keyword arguments: {keys:?}"),
                        span,
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
            interp.pop_frame();
            result
        }
        Value::BoundMethod(receiver, method_name) => {
             // Push stack frame
            interp.push_frame(&method_name, span);
            let res = {
                // Check if receiver is Foreign
                if let Value::Foreign(foreign) = receiver.as_ref() {
                    foreign
                        .call_method(&method_name, args_slice, &kw_args_val)
                        .map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()))
                } else {
                    if !kw_args_val.is_empty() {
                        Err(EldritchError::new(EldritchErrorKind::TypeError, "BoundMethod does not accept keyword arguments", span).with_stack(interp.call_stack.clone()))
                    } else {
                        call_bound_method(&receiver, &method_name, args_slice)
                            .map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()))
                    }
                }
            };
            interp.pop_frame();
            res
        }
        _ => interp.error(EldritchErrorKind::TypeError, &format!("Cannot call value of type: {callee_val:?}"), span),
    }
}

fn builtin_map(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 2 {
        return interp.error(EldritchErrorKind::TypeError, "map() takes exactly 2 arguments", span);
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
        return interp.error(EldritchErrorKind::TypeError, "filter() takes exactly 2 arguments", span);
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
        return interp.error(EldritchErrorKind::TypeError, "reduce() takes 2 or 3 arguments", span);
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let mut items = to_iterable(interp, &iterable_val, span)?.into_iter();

    let mut acc = if args.len() == 3 {
        evaluate_arg(interp, &args[2])?
    } else {
        match items.next() {
            Some(v) => v,
            None => return interp.error(EldritchErrorKind::TypeError, "reduce() of empty sequence with no initial value", span),
        }
    };

    for item in items {
        acc = call_value(interp, &func_val, &[acc, item], span)?;
    }
    Ok(acc)
}

fn builtin_sorted(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    let mut iterable_arg: Option<&Expr> = None;
    let mut key_arg: Option<&Expr> = None;
    let mut reverse_arg: Option<&Expr> = None;

    for arg in args {
        match arg {
            Argument::Positional(e) => {
                if iterable_arg.is_none() {
                    iterable_arg = Some(e);
                } else {
                    return interp.error(EldritchErrorKind::TypeError, "sorted() takes only 1 positional argument", span);
                }
            }
            Argument::Keyword(name, e) => match name.as_str() {
                "key" => key_arg = Some(e),
                "reverse" => reverse_arg = Some(e),
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &format!("sorted() got an unexpected keyword argument '{name}'"),
                        span,
                    )
                }
            },
            _ => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "sorted() does not support *args or **kwargs unpacking",
                    span,
                )
            }
        }
    }

    let iterable_expr = iterable_arg.ok_or_else(|| {
        interp.error::<()>(EldritchErrorKind::TypeError, "sorted() missing 1 required positional argument: 'iterable'", span).unwrap_err()
    })?;

    let iterable_val = evaluate(interp, iterable_expr)?;
    let mut items = to_iterable(interp, &iterable_val, span)?;

    // Handle key
    if let Some(key_expr) = key_arg {
        let key_func = evaluate(interp, key_expr)?;
        if matches!(key_func, Value::None) {
            // sort normally
            items.sort();
        } else {
            // Decorated sort
            let mut decorated = Vec::with_capacity(items.len());
            for item in items.iter() {
                let k = call_value(interp, &key_func, core::slice::from_ref(item), span)?;
                decorated.push((k, item.clone()));
            }
            // Sort decorated
            // Value implements Ord
            decorated.sort_by(|a, b| a.0.cmp(&b.0));

            // Undecorate
            items = decorated.into_iter().map(|(_, v)| v).collect();
        }
    } else {
        items.sort();
    }

    // Handle reverse
    if let Some(rev_expr) = reverse_arg {
        let rev_val = evaluate(interp, rev_expr)?;
        if is_truthy(&rev_val) {
            items.reverse();
        }
    }

    Ok(Value::List(Arc::new(RwLock::new(items))))
}

fn builtin_eval(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 1 {
        return interp.error(EldritchErrorKind::TypeError, "eval() takes exactly 1 argument", span);
    }

    let code_val = evaluate_arg(interp, &args[0])?;
    let code = match code_val {
        Value::String(s) => s,
        _ => return interp.error(EldritchErrorKind::TypeError, "eval() argument must be a string", span),
    };

    if interp.depth >= MAX_RECURSION_DEPTH {
        return interp.error(EldritchErrorKind::RecursionError, "Recursion limit exceeded", span);
    }

    // Create a new interpreter instance that shares the environment
    // We manually construct it to avoid re-loading builtins and to set depth
    let mut temp_interp = Interpreter {
        env: interp.env.clone(),
        flow: Flow::Next,
        depth: interp.depth + 1,
        call_stack: interp.call_stack.clone(),
        current_func_name: "<eval>".to_string(),
    };

    match temp_interp.interpret(&code) {
        Ok(v) => Ok(v),
        Err(e) => interp.error(EldritchErrorKind::RuntimeError, &e, span),
    }
}

fn call_value(
    interp: &mut Interpreter,
    func: &Value,
    args: &[Value],
    span: Span,
) -> Result<Value, EldritchError> {
    match func {
        Value::NativeFunction(_, f) => {
             // Push stack frame
             // Native function name?
             interp.push_frame("<native>", span);
            let res = f(&interp.env, args).map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()));
            interp.pop_frame();
            res
        }
        Value::Function(Function {
            name: _,
            params: _,
            body: _,
            closure: _,
        }) => {
            if interp.depth >= MAX_RECURSION_DEPTH {
                return interp.error(EldritchErrorKind::RecursionError, "Recursion limit exceeded", span);
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
        Value::BoundMethod(receiver, method_name) => {
             interp.push_frame(method_name, span);
             let res = call_bound_method(receiver, method_name, args)
                .map_err(|e| EldritchError::new(EldritchErrorKind::RuntimeError, &e, span).with_stack(interp.call_stack.clone()));
             interp.pop_frame();
             res
        },
        _ => interp.error(EldritchErrorKind::TypeError, "not callable", span),
    }
}

fn evaluate_arg(interp: &mut Interpreter, arg: &Argument) -> Result<Value, EldritchError> {
    match arg {
        Argument::Positional(e) => evaluate(interp, e),
        // Just return a dummy span here for the error, or match e.span if available.
        // Since we don't have easy access to a span here without unpacking, use a dummy one.
        _ => interp.error(
            EldritchErrorKind::TypeError,
            "HOFs currently only support positional arguments",
            Span::new(0, 0, 0),
        ),
    }
}

pub(crate) fn to_iterable(
    interp: &Interpreter,
    val: &Value,
    span: Span,
) -> Result<Vec<Value>, EldritchError> {
    match val {
        Value::List(l) => Ok(l.read().clone()),
        Value::Tuple(t) => Ok(t.clone()),
        Value::Set(s) => Ok(s.read().iter().cloned().collect()),
        Value::Dictionary(d) => Ok(d.read().keys().cloned().collect()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("Type '{:?}' is not iterable", get_type_name(val)),
            span,
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
            _ => interp.error(EldritchErrorKind::TypeError, "Unary '-' only valid for numbers", span),
        },
        TokenKind::Not => Ok(Value::Bool(!is_truthy(&val))),
        TokenKind::BitNot => match val {
            Value::Int(i) => Ok(Value::Int(!i)),
            _ => interp.error(EldritchErrorKind::TypeError, "Bitwise '~' only valid for integers", span),
        },
        _ => interp.error(EldritchErrorKind::SyntaxError, "Invalid unary operator", span),
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
        _ => interp.error(EldritchErrorKind::SyntaxError, "Invalid logical operator", span),
    }
}

fn evaluate_in(
    interp: &mut Interpreter,
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
            Ok(Value::Bool(dict.contains_key(item)))
        }
        Value::Set(s) => {
            let set = s.read();
            Ok(Value::Bool(set.contains(item)))
        }
        Value::String(s) => {
            let sub = match item {
                Value::String(ss) => ss,
                _ => return interp.error(EldritchErrorKind::TypeError, "'in <string>' requires string as left operand", span),
            };
            Ok(Value::Bool(s.contains(sub)))
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "argument of type '{}' is not iterable",
                get_type_name(collection)
            ),
            span,
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

    // Handle operations that are fully delegated
    if matches!(
        op,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::SlashSlash
            | TokenKind::Percent
    ) {
        // Some specific type combinations need special handling, but generic arithmetic is in helper.
        // Wait, string concatenation is + but logic is in apply_binary_op?
        // Let's check which are generic.

        // Only numbers are fully handled in apply_arithmetic_op
        if (matches!(a, Value::Int(_) | Value::Float(_))
            && matches!(b, Value::Int(_) | Value::Float(_)))
        {
            return apply_arithmetic_op(interp, &a, op, &b, span);
        }
    }

    // Comparisons
    if matches!(
        op,
        TokenKind::Eq
            | TokenKind::NotEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
    ) {
        // Sequence comparison is special (recursive).
        // Numbers and Mixed are handled in helper.
        match (&a, &b) {
            (Value::List(la), Value::List(lb)) => {
                let list_a = la.read();
                let list_b = lb.read();
                return compare_sequences(&list_a, &list_b, op.clone(), span);
            }
            (Value::Tuple(ta), Value::Tuple(tb)) => {
                return compare_sequences(ta, tb, op.clone(), span);
            }
            _ => return apply_comparison_op(interp, &a, op, &b, span),
        }
    }

    // Bitwise
    if matches!(
        op,
        TokenKind::BitAnd
            | TokenKind::BitOr
            | TokenKind::BitXor
            | TokenKind::LShift
            | TokenKind::RShift
    ) {
        return apply_bitwise_op(interp, &a, op, &b, span);
    }

    match (a, op.clone(), b) {
        // IN Operator
        (item, TokenKind::In, collection) => evaluate_in(interp, &item, &collection, span),
        (item, TokenKind::NotIn, collection) => {
            let res = evaluate_in(interp, &item, &collection, span)?;
            match res {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => unreachable!("evaluate_in always returns boolean or error"),
            }
        }

        // Non-numeric arithmetic (Sequences)

        // Set Difference (Minus)
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
        (Value::List(mut a), TokenKind::Plus, Value::List(b)) => {
            // Optimization: If `a` is a temporary (unique), mutate in place
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                let list = rw_lock.get_mut();
                list.extend(b.read().clone());
                Ok(Value::List(a))
            } else {
                let mut new_list = a.read().clone();
                new_list.extend(b.read().clone());
                Ok(Value::List(Arc::new(RwLock::new(new_list))))
            }
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

        // Dict merge (new dict)
        (Value::Dictionary(mut a), TokenKind::Plus, Value::Dictionary(b)) => {
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                let dict = rw_lock.get_mut();
                for (k, v) in b.read().iter() {
                    dict.insert(k.clone(), v.clone());
                }
                Ok(Value::Dictionary(a))
            } else {
                let mut new_dict = a.read().clone();
                for (k, v) in b.read().iter() {
                    new_dict.insert(k.clone(), v.clone());
                }
                Ok(Value::Dictionary(Arc::new(RwLock::new(new_dict))))
            }
        }

        // Set union (new set) - Plus is deprecated for sets in favor of |
        (Value::Set(mut a), TokenKind::Plus, Value::Set(b)) => {
            if let Some(rw_lock) = Arc::get_mut(&mut a) {
                #[allow(clippy::mutable_key_type)]
                let set = rw_lock.get_mut();
                for item in b.read().iter() {
                    set.insert(item.clone());
                }
                Ok(Value::Set(a))
            } else {
                #[allow(clippy::mutable_key_type)]
                let mut new_set = a.read().clone();
                for item in b.read().iter() {
                    new_set.insert(item.clone());
                }
                Ok(Value::Set(Arc::new(RwLock::new(new_set))))
            }
        }

        _ => interp.error(EldritchErrorKind::TypeError, "Unsupported binary op", span),
    }
}

fn string_modulo_format(
    interp: &mut Interpreter,
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
                // If it is '%%', handle immediately
                if next == '%' {
                    chars.next();
                    result.push('%');
                    continue;
                }

                chars.next(); // Consume specifier

                if val_idx >= vals.len() {
                    return interp.error(EldritchErrorKind::TypeError, "not enough arguments for format string", span);
                }
                let v = &vals[val_idx];
                val_idx += 1;

                match next {
                    's' => {
                        result.push_str(&v.to_string());
                    }
                    'd' | 'i' | 'u' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val}");
                    }
                    'o' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:o}");
                    }
                    'x' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:x}");
                    }
                    'X' => {
                        let i_val = to_int_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{i_val:X}");
                    }
                    'e' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:e}");
                    }
                    'E' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:E}");
                    }
                    'f' | 'F' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:.6}",);
                    }
                    'g' | 'G' => {
                        let f_val = to_float_or_error(interp, v, next, span)?;
                        let _ = write!(result, "{f_val:?}");
                    }
                    'r' => match v {
                        Value::String(s) => {
                            let _ = write!(result, "\"{s}\"");
                        }
                        _ => result.push_str(&v.to_string()),
                    },
                    _ => {
                        return interp.error(
                            EldritchErrorKind::ValueError,
                            &format!("Unsupported format specifier: %{next}"),
                            span,
                        );
                    }
                }
            } else {
                return interp.error(EldritchErrorKind::ValueError, "incomplete format", span);
            }
        } else {
            result.push(c);
        }
    }

    if val_idx < vals.len() {
        return interp.error(EldritchErrorKind::TypeError, "not all arguments converted during string formatting", span);
    }

    Ok(Value::String(result))
}

fn to_int_or_error(interp: &Interpreter, v: &Value, spec: char, span: Span) -> Result<i64, EldritchError> {
    match v {
        Value::Int(i) => Ok(*i),
        Value::Float(f) => Ok(*f as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "%{} format: a number is required, not {:?}",
                spec,
                get_type_name(v)
            ),
            span,
        ),
    }
}

fn to_float_or_error(interp: &Interpreter, v: &Value, spec: char, span: Span) -> Result<f64, EldritchError> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "%{} format: a number is required, not {:?}",
                spec,
                get_type_name(v)
            ),
            span,
        ),
    }
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
                    Ok(Value::Bool(
                        super::operations::compare_values(val_a, val_b)
                            .is_ok_and(|ord| matches!(ord, core::cmp::Ordering::Less)),
                    ))
                }
                TokenKind::Gt => Ok(Value::Bool(
                    super::operations::compare_values(val_a, val_b)
                        .is_ok_and(|ord| matches!(ord, core::cmp::Ordering::Greater)),
                )),
                TokenKind::LtEq => Ok(Value::Bool(
                    super::operations::compare_values(val_a, val_b).is_ok_and(|ord| {
                        matches!(ord, core::cmp::Ordering::Less | core::cmp::Ordering::Equal)
                    }),
                )),
                TokenKind::GtEq => Ok(Value::Bool(
                    super::operations::compare_values(val_a, val_b).is_ok_and(|ord| {
                        matches!(
                            ord,
                            core::cmp::Ordering::Greater | core::cmp::Ordering::Equal
                        )
                    }),
                )),
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
