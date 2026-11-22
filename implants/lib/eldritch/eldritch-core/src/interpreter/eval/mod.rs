pub mod access;
pub mod comprehensions;
pub mod functions;
pub mod literals;
pub mod ops;
pub mod strings;
pub mod utils;

use super::super::ast::{Expr, ExprKind, Value};
use super::core::Interpreter;
use super::error::{EldritchError, EldritchErrorKind};
use super::introspection::is_truthy;

use self::access::{evaluate_getattr, evaluate_index, evaluate_slice};
use self::comprehensions::{evaluate_dict_comp, evaluate_list_comp, evaluate_set_comp};
use self::functions::{call_function, evaluate_lambda};
use self::literals::{
    evaluate_dict_literal, evaluate_list_literal, evaluate_set_literal, evaluate_tuple_literal,
};
use self::ops::{apply_binary_op, apply_logical_op, apply_unary_op};
use self::strings::evaluate_fstring;
use alloc::format;

// Re-export for exec.rs
pub(crate) use self::ops::apply_binary_op as apply_binary_op_pub;
pub(crate) use self::utils::to_iterable;

pub(crate) const MAX_RECURSION_DEPTH: usize = 64;
pub(crate) const MAX_EXPR_DEPTH: usize = 128;

pub fn evaluate(interp: &mut Interpreter, expr: &Expr) -> Result<Value, EldritchError> {
    if interp.expr_depth >= MAX_EXPR_DEPTH {
        return interp.error(
            EldritchErrorKind::RecursionError,
            "Maximum expression evaluation depth exceeded",
            expr.span,
        );
    }
    interp.expr_depth += 1;
    let res = evaluate_inner(interp, expr);
    interp.expr_depth -= 1;
    res
}

fn evaluate_inner(interp: &mut Interpreter, expr: &Expr) -> Result<Value, EldritchError> {
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
        ExprKind::GetAttr(obj, name) => evaluate_getattr(interp, obj, name.clone()),
        ExprKind::Slice(obj, start, stop, step) => {
            evaluate_slice(interp, obj, start, stop, step, span)
        }
        ExprKind::FString(segments) => evaluate_fstring(interp, segments),
        ExprKind::ListComp {
            body,
            vars,
            iterable,
            cond,
        } => evaluate_list_comp(interp, body, vars, iterable, cond),
        ExprKind::DictComp {
            key,
            value,
            vars,
            iterable,
            cond,
        } => evaluate_dict_comp(interp, key, value, vars, iterable, cond),
        ExprKind::SetComp {
            body,
            vars,
            iterable,
            cond,
        } => evaluate_set_comp(interp, body, vars, iterable, cond),
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
        ExprKind::Error(msg) => interp.runtime_error(
            &format!("Runtime encountered syntax error node: {}", msg),
            span,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Span, TokenKind};
    use alloc::boxed::Box;

    #[test]
    fn test_evaluate_expr_depth_limit() {
        let span = Span::new(0, 1, 1);
        let mut expr = Expr {
            kind: ExprKind::Literal(Value::Int(1)),
            span,
        };
        for _ in 0..(MAX_EXPR_DEPTH + 10) {
            expr = Expr {
                kind: ExprKind::UnaryOp(TokenKind::Minus, Box::new(expr)),
                span,
            };
        }
        let mut interp = Interpreter::new();
        let err = evaluate(&mut interp, &expr).unwrap_err();
        assert_eq!(err.kind, EldritchErrorKind::RecursionError);
    }

    #[test]
    fn test_unary_neg_i64_min_returns_runtime_error() {
        let span = Span::new(0, 1, 1);
        let expr = Expr {
            kind: ExprKind::UnaryOp(
                TokenKind::Minus,
                Box::new(Expr {
                    kind: ExprKind::Literal(Value::Int(i64::MIN)),
                    span,
                }),
            ),
            span,
        };
        let mut interp = Interpreter::new();
        let err = evaluate(&mut interp, &expr).unwrap_err();
        assert_eq!(err.kind, EldritchErrorKind::RuntimeError);
    }
}
