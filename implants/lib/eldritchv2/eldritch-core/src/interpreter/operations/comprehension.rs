use crate::ast::{Environment, Expr};
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::EldritchError;
use crate::interpreter::eval::evaluate;
use crate::interpreter::introspection::is_truthy;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use spin::RwLock;

pub(crate) fn evaluate_comprehension_generic<F>(
    interp: &mut Interpreter,
    var: &str,
    iterable: &Expr,
    cond: &Option<Box<Expr>>,
    mut insert_fn: F,
) -> Result<(), EldritchError>
where
    F: FnMut(&mut Interpreter) -> Result<(), EldritchError>,
{
    let iterable_val = evaluate(interp, iterable)?;
    // Use crate::interpreter::eval::to_iterable
    let items = crate::interpreter::eval::to_iterable(interp, &iterable_val, iterable.span)?;

    let printer = interp.env.read().printer.clone();
    let comp_env = Arc::new(RwLock::new(Environment {
        parent: Some(interp.env.clone()),
        values: BTreeMap::new(),
        printer,
        libraries: BTreeSet::new(),
    }));
    let original_env = interp.env.clone();
    interp.env = comp_env;

    for item in items {
        interp.define_variable(var, item);
        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            insert_fn(interp)?;
        }
    }
    interp.env = original_env;
    Ok(())
}
