use super::super::ast::{
    Environment, Expr, ExprKind, Function, Param, RuntimeParam, Stmt, StmtKind, Value,
};
use super::super::token::TokenKind;
use super::core::{Flow, Interpreter};
use super::error::{runtime_error, EldritchError};
use super::eval::{apply_binary_op_pub, evaluate};
use super::utils::{get_type_name, is_truthy};
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn execute(interp: &mut Interpreter, stmt: &Stmt) -> Result<(), EldritchError> {
    if interp.flow != Flow::Next {
        return Ok(());
    }

    match &stmt.kind {
        StmtKind::Expression(expr) => {
            evaluate(interp, expr)?;
        }
        StmtKind::Assignment(target_expr, value_expr) => {
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
        StmtKind::Def(name, params, body) => {
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

            let func = Value::Function(Function {
                name: name.clone(),
                params: runtime_params,
                body: body.clone(),
                closure: interp.env.clone(),
            });
            interp.env.borrow_mut().values.insert(name.clone(), func);
        }
        StmtKind::For(idents, iterable, body) => {
            let iterable_val = evaluate(interp, iterable)?;
            let items: Vec<Value> = match iterable_val {
                Value::List(l) => l.borrow().clone(),
                Value::Tuple(t) => t.clone(),
                Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                Value::Bytes(b) => b.iter().map(|&byte| Value::Int(byte as i64)).collect(),
                _ => return runtime_error(
                    iterable.span,
                    &format!(
                        "'for' loop can only iterate over lists/iterables. Found {iterable_val:?}"
                    ),
                ),
            };

            for item in items {
                // Scope per iteration to prevent leaking variables
                let parent_env = Rc::clone(&interp.env);
                let printer = parent_env.borrow().printer.clone();
                let new_env = Rc::new(RefCell::new(Environment {
                    parent: Some(parent_env.clone()),
                    values: BTreeMap::new(),
                    printer,
                }));
                interp.env = new_env;

                // Use define_variable to force loop variables into the new scope
                if idents.len() == 1 {
                    interp.define_variable(&idents[0], item);
                } else {
                    let parts = match item {
                        Value::List(l) => l.borrow().clone(),
                        Value::Tuple(t) => t.clone(),
                        _ => {
                            interp.env = parent_env;
                            return runtime_error(stmt.span, "Cannot unpack non-iterable");
                        }
                    };

                    if parts.len() != idents.len() {
                        interp.env = parent_env;
                        return runtime_error(stmt.span, &format!("ValueError: too many/not enough values to unpack (expected {}, got {})", idents.len(), parts.len()));
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
                            return runtime_error(index_expr.span, "List indices must be integers")
                        }
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
                        _ => {
                            return runtime_error(
                                index_expr.span,
                                "Dictionary keys must be strings",
                            )
                        }
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
                            return runtime_error(index_expr.span, "List indices must be integers")
                        }
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
                        _ => {
                            return runtime_error(
                                index_expr.span,
                                "Dictionary keys must be strings",
                            )
                        }
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
