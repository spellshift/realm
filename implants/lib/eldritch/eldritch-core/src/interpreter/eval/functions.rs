use super::super::super::ast::{
    Argument, Environment, Expr, ExprKind, Function, Param, RuntimeParam, Stmt, StmtKind, Value,
};
use super::super::super::token::Span;
use super::super::builtins::{
    eval_builtin::builtin_eval_func, filter::builtin_filter, map::builtin_map,
    reduce::builtin_reduce, sorted::builtin_sorted,
};
use super::super::core::{Flow, Interpreter};
use super::super::error::{EldritchError, EldritchErrorKind};
use super::super::exec::execute_stmts;
use super::super::introspection::get_type_name;
use super::super::methods::call_bound_method;
use super::utils::parse_error_kind;
use super::{MAX_RECURSION_DEPTH, evaluate};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub(crate) fn evaluate_lambda(
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
        body: alloc::vec![ret_stmt],
        closure: interp.env.clone(),
    });
    Ok(func)
}

pub(crate) fn call_function(
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
            return builtin_eval_func(interp, args, span);
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
                        );
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
                                _ => {
                                    return interp.error(
                                        EldritchErrorKind::TypeError,
                                        "Keywords must be strings",
                                        expr.span,
                                    );
                                }
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
                        );
                    }
                }
            }
        }
    }

    let args_slice = pos_args_val.as_slice();

    match callee_val {
        Value::NativeFunction(_, f) => {
            if !kw_args_val.is_empty() {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "NativeFunction does not accept keyword arguments",
                    span,
                );
            }
            // Ensure stack frame for native call
            // Native function name?
            if let ExprKind::Identifier(name) = &callee.kind {
                interp.push_frame(name, span);
            } else {
                interp.push_frame("<native>", span);
            }

            let res = f(&interp.env, args_slice).map_err(|e| {
                let (kind, msg) = parse_error_kind(&e);
                EldritchError::new(kind, msg, span).with_stack(interp.call_stack.clone())
            });
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
            let res = f(&interp.env, args_slice, &kw_args_val).map_err(|e| {
                let (kind, msg) = parse_error_kind(&e);
                EldritchError::new(kind, msg, span).with_stack(interp.call_stack.clone())
            });
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
                return interp.error(
                    EldritchErrorKind::RecursionError,
                    "Recursion limit exceeded",
                    span,
                );
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
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        "Function got too many positional arguments.",
                        span,
                    );
                }

                if !kw_args_val.is_empty() {
                    let mut keys: Vec<&String> = kw_args_val.keys().collect();
                    keys.sort();
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &format!(
                            "{}() got an unexpected keyword argument '{}'",
                            name, keys[0]
                        ),
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
                        .call_method(interp, &method_name, args_slice, &kw_args_val)
                        .map_err(|e| {
                            let (kind, msg) = parse_error_kind(&e);
                            EldritchError::new(kind, msg, span)
                                .with_stack(interp.call_stack.clone())
                        })
                } else if !kw_args_val.is_empty() {
                    Err(EldritchError::new(
                        EldritchErrorKind::TypeError,
                        "BoundMethod does not accept keyword arguments",
                        span,
                    )
                    .with_stack(interp.call_stack.clone()))
                } else {
                    call_bound_method(&receiver, &method_name, args_slice).map_err(|e| {
                        let (kind, msg) = parse_error_kind(&e);
                        EldritchError::new(kind, msg, span).with_stack(interp.call_stack.clone())
                    })
                }
            };
            interp.pop_frame();
            res
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("Cannot call value of type: {callee_val:?}"),
            span,
        ),
    }
}

pub(crate) fn call_value(
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
            let res = f(&interp.env, args).map_err(|e| {
                let (kind, msg) = parse_error_kind(&e);
                EldritchError::new(kind, msg, span).with_stack(interp.call_stack.clone())
            });
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
                return interp.error(
                    EldritchErrorKind::RecursionError,
                    "Recursion limit exceeded",
                    span,
                );
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
            let res = call_bound_method(receiver, method_name, args).map_err(|e| {
                let (kind, msg) = parse_error_kind(&e);
                EldritchError::new(kind, msg, span).with_stack(interp.call_stack.clone())
            });
            interp.pop_frame();
            res
        }
        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!("'{}' object is not callable", get_type_name(func)),
            span,
        ),
    }
}

pub(crate) fn evaluate_arg(
    interp: &mut Interpreter,
    arg: &Argument,
) -> Result<Value, EldritchError> {
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
