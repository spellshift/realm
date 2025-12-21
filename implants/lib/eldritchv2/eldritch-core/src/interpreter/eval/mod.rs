pub mod access;
pub mod comprehensions;
pub mod functions;
pub mod literals;
pub mod ops;
pub mod strings;
pub mod utils;

use super::super::ast::{Expr, ExprKind, Value};
use super::core::Interpreter;
use super::error::EldritchError;
use super::introspection::is_truthy;

use self::access::{evaluate_getattr, evaluate_index, evaluate_slice};
use self::comprehensions::{evaluate_dict_comp, evaluate_list_comp, evaluate_set_comp};
use self::functions::{call_function, evaluate_lambda};
use self::literals::{
    evaluate_dict_literal, evaluate_list_literal, evaluate_set_literal, evaluate_tuple_literal,
};
use self::ops::{apply_binary_op, apply_logical_op, apply_unary_op};
use self::strings::evaluate_fstring;

// Re-export for exec.rs
pub(crate) use self::ops::apply_binary_op as apply_binary_op_pub;
pub(crate) use self::utils::to_iterable;

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
        ExprKind::GetAttr(obj, name) => evaluate_getattr(interp, obj, name.clone()),
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
