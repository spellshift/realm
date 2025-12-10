use super::super::ast::{
    Environment, Expr, ExprKind, Function, Param, RuntimeParam, Stmt, StmtKind, Value,
};
use super::super::token::TokenKind;
use super::core::{Flow, Interpreter};
use super::error::{EldritchError, EldritchErrorKind};
use super::eval::{apply_binary_op_pub, evaluate};
use super::introspection::{get_type_name, is_truthy};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub fn execute(interp: &mut Interpreter, stmt: &Stmt) -> Result<(), EldritchError> {
    if interp.flow != Flow::Next {
        return Ok(());
    }

    match &stmt.kind {
        StmtKind::Expression(expr) => {
            evaluate(interp, expr)?;
        }
        StmtKind::Assignment(target_expr, _annotation, value_expr) => {
            let value = evaluate(interp, value_expr)?;
            assign(interp, target_expr, value)?;
        }
        StmtKind::AugmentedAssignment(target_expr, op, value_expr) => {
            execute_augmented_assignment(interp, target_expr, op, value_expr)?;
        }
        StmtKind::If(condition, then_branch, else_branch) => {
            let eval_cond = &evaluate(interp, condition)?;
            if is_truthy(eval_cond) {
                execute_stmts(interp, then_branch)?;
            } else if let Some(else_stmts) = else_branch {
                execute_stmts(interp, else_stmts)?;
            }
        }
        StmtKind::Return(expr) => {
            let val = expr
                .as_ref()
                .map_or(Ok(Value::None), |e| evaluate(interp, e))?;
            interp.flow = Flow::Return(val);
        }
        StmtKind::Def(name, params, _return_annotation, body) => {
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

            let func = Value::Function(Function {
                name: name.clone(),
                params: runtime_params,
                body: body.clone(),
                closure: interp.env.clone(),
            });
            interp.env.write().values.insert(name.clone(), func);
        }
        StmtKind::For(idents, iterable, body) => {
            let iterable_val = evaluate(interp, iterable)?;
            let items: Vec<Value> = match iterable_val {
                Value::List(l) => l.read().clone(),
                Value::Tuple(t) => t.clone(),
                Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                Value::Bytes(b) => b.iter().map(|&byte| Value::Int(byte as i64)).collect(),
                Value::Dictionary(d) => d.read().keys().cloned().collect(),
                _ => return interp.error(
                    EldritchErrorKind::TypeError,
                    &format!(
                        "'for' loop can only iterate over lists/iterables. Found {iterable_val:?}"
                    ),
                    iterable.span,
                ),
            };

            for item in items {
                // Scope per iteration to prevent leaking variables
                let parent_env = interp.env.clone();
                let printer = parent_env.read().printer.clone();
                let new_env = Arc::new(RwLock::new(Environment {
                    parent: Some(parent_env.clone()),
                    values: BTreeMap::new(),
                    printer,
                    libraries: BTreeSet::new(),
                }));
                interp.env = new_env;

                // Use define_variable to force loop variables into the new scope
                if idents.len() == 1 {
                    interp.define_variable(&idents[0], item);
                } else {
                    let parts = match item {
                        Value::List(l) => l.read().clone(),
                        Value::Tuple(t) => t.clone(),
                        _ => {
                            interp.env = parent_env;
                            return interp.error(EldritchErrorKind::TypeError, "Cannot unpack non-iterable", stmt.span);
                        }
                    };

                    if parts.len() != idents.len() {
                        interp.env = parent_env;
                        return interp.error(EldritchErrorKind::ValueError, &format!("ValueError: too many/not enough values to unpack (expected {}, got {})", idents.len(), parts.len()), stmt.span);
                    }

                    for (var, val) in idents.iter().zip(parts.into_iter()) {
                        interp.define_variable(var, val);
                    }
                }

                let result = execute_stmts(interp, body);
                interp.env = parent_env;
                result?;

                match &interp.flow {
                    Flow::Break => {
                        interp.flow = Flow::Next;
                        break;
                    }
                    Flow::Continue => {
                        interp.flow = Flow::Next;
                        continue;
                    }
                    Flow::Return(_) => return Ok(()),
                    Flow::Next => {}
                }
            }
        }
        StmtKind::Break => interp.flow = Flow::Break,
        StmtKind::Continue => interp.flow = Flow::Continue,
        StmtKind::Pass => {} // Do nothing
    }
    Ok(())
}

pub fn execute_stmts(interp: &mut Interpreter, stmts: &[Stmt]) -> Result<(), EldritchError> {
    for stmt in stmts {
        execute(interp, stmt)?;
        if interp.flow != Flow::Next {
            break;
        }
    }
    Ok(())
}

fn assign(interp: &mut Interpreter, target: &Expr, value: Value) -> Result<(), EldritchError> {
    match &target.kind {
        ExprKind::Identifier(name) => {
            interp.assign_variable(name, value);
            Ok(())
        }
        ExprKind::List(elements) | ExprKind::Tuple(elements) => {
            // Unpacking
            let values = match value {
                Value::List(l) => l.read().clone(),
                Value::Tuple(t) => t.clone(),
                _ => {
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &format!("cannot unpack non-iterable {:?}", get_type_name(&value)),
                        target.span,
                    )
                }
            };

            if elements.len() != values.len() {
                return interp.error(
                    EldritchErrorKind::ValueError,
                    &format!(
                        "ValueError: too many/not enough values to unpack (expected {}, got {})",
                        elements.len(),
                        values.len()
                    ),
                    target.span,
                );
            }

            for (target_elem, val_elem) in elements.iter().zip(values.into_iter()) {
                assign(interp, target_elem, val_elem)?;
            }
            Ok(())
        }
        ExprKind::Index(obj_expr, index_expr) => {
            let obj = evaluate(interp, obj_expr)?;
            let index = evaluate(interp, index_expr)?;
            match obj {
                Value::List(l) => {
                    let idx_int = match index {
                        Value::Int(i) => i,
                        _ => {
                            return interp.error(EldritchErrorKind::TypeError, "List indices must be integers", index_expr.span)
                        }
                    };
                    let mut list = l.write();
                    let true_idx = if idx_int < 0 {
                        list.len() as i64 + idx_int
                    } else {
                        idx_int
                    };
                    if true_idx < 0 || true_idx as usize >= list.len() {
                        return interp.error(EldritchErrorKind::IndexError, "List assignment index out of range", target.span);
                    }
                    list[true_idx as usize] = value;
                    Ok(())
                }
                Value::Dictionary(d) => {
                    d.write().insert(index, value);
                    Ok(())
                }
                _ => interp.error(EldritchErrorKind::TypeError, "Object does not support item assignment", target.span),
            }
        }
        _ => interp.error(EldritchErrorKind::SyntaxError, "cannot assign to this expression", target.span),
    }
}

fn execute_augmented_assignment(
    interp: &mut Interpreter,
    target: &Expr,
    op: &TokenKind,
    value_expr: &Expr,
) -> Result<(), EldritchError> {
    let span = target.span;
    let right = evaluate(interp, value_expr)?;

    // For simple identifiers, read, op, write
    match &target.kind {
        ExprKind::Identifier(name) => {
            let left = interp.lookup_variable(name, span)?;

            if let TokenKind::PlusAssign = op {
                if try_inplace_add(&left, &right) {
                    return Ok(());
                }
            }

            let bin_op = augmented_op_to_binary(op).ok_or_else(|| {
                EldritchError {
                    span,
                    message: "Unknown augmented assignment operator".to_string(),
                    kind: EldritchErrorKind::SyntaxError,
                    stack: Vec::new(),
                }
            })?;

            // Construct dummy expressions for apply_binary_op call to reuse logic
            let left_expr = Expr {
                kind: ExprKind::Literal(left),
                span,
            };
            let right_expr = Expr {
                kind: ExprKind::Literal(right),
                span,
            };

            let new_val = apply_binary_op_pub(interp, &left_expr, &bin_op, &right_expr, span)?;
            interp.assign_variable(name, new_val);
            Ok(())
        }
        ExprKind::Index(obj_expr, index_expr) => {
            let obj = evaluate(interp, obj_expr)?;
            let index = evaluate(interp, index_expr)?;

            // This is tricky: we need to get the item, op it, and set it back.
            // For mutable objects (List, Dict), we can modify in place or set item.

            let current_val = match &obj {
                Value::List(l) => {
                    let idx_int = match index {
                        Value::Int(i) => i,
                        _ => {
                            return interp.error(EldritchErrorKind::TypeError, "List indices must be integers", index_expr.span)
                        }
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
                    list[true_idx as usize].clone()
                }
                Value::Dictionary(d) => {
                    let dict = d.read();
                    match dict.get(&index) {
                        Some(v) => v.clone(),
                        None => return interp.error(EldritchErrorKind::KeyError, "KeyError", span),
                    }
                }
                _ => return interp.error(EldritchErrorKind::TypeError, "Object does not support item assignment", span),
            };

            if let TokenKind::PlusAssign = op {
                if try_inplace_add(&current_val, &right) {
                    return Ok(());
                }
            }

            let bin_op = augmented_op_to_binary(op).ok_or_else(|| {
                EldritchError {
                    span,
                    message: "Unknown augmented assignment operator".to_string(),
                    kind: EldritchErrorKind::SyntaxError,
                    stack: Vec::new(),
                }
            })?;

            let left_expr = Expr {
                kind: ExprKind::Literal(current_val),
                span,
            };
            let right_expr = Expr {
                kind: ExprKind::Literal(right),
                span,
            };
            let new_val = apply_binary_op_pub(interp, &left_expr, &bin_op, &right_expr, span)?;

            // Set back
            match obj {
                Value::List(l) => {
                    // Need to re-calculate index as borrow ends
                    let idx_int = match index {
                        Value::Int(i) => i,
                        _ => unreachable!(),
                    };
                    let mut list = l.write();
                    let true_idx = if idx_int < 0 {
                        list.len() as i64 + idx_int
                    } else {
                        idx_int
                    };
                    list[true_idx as usize] = new_val;
                    Ok(())
                }
                Value::Dictionary(d) => {
                    d.write().insert(index, new_val);
                    Ok(())
                }
                _ => unreachable!(),
            }
        }
        _ => interp.error(EldritchErrorKind::SyntaxError, "Illegal target for augmented assignment", span),
    }
}

fn augmented_op_to_binary(op: &TokenKind) -> Option<TokenKind> {
    match op {
        TokenKind::PlusAssign => Some(TokenKind::Plus),
        TokenKind::MinusAssign => Some(TokenKind::Minus),
        TokenKind::StarAssign => Some(TokenKind::Star),
        TokenKind::SlashAssign => Some(TokenKind::Slash),
        TokenKind::SlashSlashAssign => Some(TokenKind::SlashSlash),
        TokenKind::PercentAssign => Some(TokenKind::Percent),
        _ => None,
    }
}

fn try_inplace_add(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::List(l), Value::List(r)) => {
            // Must clone right first to avoid double borrow panic if l == r
            if Arc::ptr_eq(l, r) {
                // Self append.
                let items = l.read().clone();
                l.write().extend(items);
            } else {
                let items = r.read().clone();
                l.write().extend(items);
            }
            true
        }
        (Value::Dictionary(d), Value::Dictionary(r)) => {
            if Arc::ptr_eq(d, r) {
                // Self merge is a no-op for dicts (keys overlap 100%).
            } else {
                let items: Vec<_> = r
                    .read()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let mut db = d.write();
                for (k, v) in items {
                    db.insert(k, v);
                }
            }
            true
        }
        (Value::Set(s), Value::Set(r)) => {
            if Arc::ptr_eq(s, r) {
                // Union with self is no-op.
            } else {
                #[allow(clippy::mutable_key_type)]
                let items: Vec<_> = r.read().iter().cloned().collect();
                #[allow(clippy::mutable_key_type)]
                let mut sb = s.write();
                for item in items {
                    sb.insert(item);
                }
            }
            true
        }
        _ => false,
    }
}
