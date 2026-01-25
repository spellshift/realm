use super::super::super::ast::{Expr, Value};
use super::super::core::Interpreter;
use super::super::error::EldritchError;
use super::evaluate;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub(crate) fn evaluate_list_literal(
    interp: &mut Interpreter,
    elements: &[Expr],
) -> Result<Value, EldritchError> {
    let mut vals = Vec::new();
    for expr in elements {
        vals.push(evaluate(interp, expr)?);
    }
    Ok(Value::List(Arc::new(RwLock::new(vals))))
}

pub(crate) fn evaluate_tuple_literal(
    interp: &mut Interpreter,
    elements: &[Expr],
) -> Result<Value, EldritchError> {
    let mut vals = Vec::new();
    for expr in elements {
        vals.push(evaluate(interp, expr)?);
    }
    Ok(Value::Tuple(vals))
}

pub(crate) fn evaluate_dict_literal(
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

pub(crate) fn evaluate_set_literal(
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
