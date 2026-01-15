use super::super::super::ast::{Expr, Value};
use super::super::core::Interpreter;
use super::super::error::EldritchError;
use super::super::operations::evaluate_comprehension_generic;
use super::evaluate;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub(crate) fn evaluate_list_comp(
    interp: &mut Interpreter,
    body: &Expr,
    vars: &[alloc::string::String],
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    let mut results = Vec::new();
    evaluate_comprehension_generic(interp, vars, iterable, cond, |i| {
        results.push(evaluate(i, body)?);
        Ok(())
    })?;
    Ok(Value::List(Arc::new(RwLock::new(results))))
}

pub(crate) fn evaluate_dict_comp(
    interp: &mut Interpreter,
    key_expr: &Expr,
    val_expr: &Expr,
    vars: &[alloc::string::String],
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    let mut results = BTreeMap::new();
    evaluate_comprehension_generic(interp, vars, iterable, cond, |i| {
        let k = evaluate(i, key_expr)?;
        let v = evaluate(i, val_expr)?;
        results.insert(k, v);
        Ok(())
    })?;
    Ok(Value::Dictionary(Arc::new(RwLock::new(results))))
}

pub(crate) fn evaluate_set_comp(
    interp: &mut Interpreter,
    body: &Expr,
    vars: &[alloc::string::String],
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
) -> Result<Value, EldritchError> {
    #[allow(clippy::mutable_key_type)]
    let mut results = BTreeSet::new();
    evaluate_comprehension_generic(interp, vars, iterable, cond, |i| {
        results.insert(evaluate(i, body)?);
        Ok(())
    })?;
    Ok(Value::Set(Arc::new(RwLock::new(results))))
}
