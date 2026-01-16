use crate::ast::{Environment, Expr};
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::eval::evaluate;
use crate::interpreter::introspection::is_truthy;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use spin::RwLock;

use crate::ast::Value;

pub(crate) fn evaluate_comprehension_generic<F>(
    interp: &mut Interpreter,
    vars: &[alloc::string::String],
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
        if vars.len() == 1 {
            interp.define_variable(&vars[0], item);
        } else {
            // Unpack
            let elements = match item {
                Value::List(l) => l.read().clone(),
                Value::Tuple(t) => t.clone(),
                Value::Set(s) => s.read().iter().cloned().collect(),
                _ => {
                    interp.env = original_env;
                    return interp.error(
                        EldritchErrorKind::TypeError,
                        &alloc::format!("Cannot unpack non-iterable object of type {}", item),
                        iterable.span,
                    );
                }
            };

            if elements.len() != vars.len() {
                interp.env = original_env;
                return interp.error(
                    EldritchErrorKind::ValueError,
                    &alloc::format!(
                        "Too many (or not enough) values to unpack (expected {}, got {})",
                        vars.len(),
                        elements.len()
                    ),
                    iterable.span,
                );
            }

            for (var, val) in vars.iter().zip(elements.into_iter()) {
                interp.define_variable(var, val);
            }
        }

        let include = match cond {
            Some(c) => is_truthy(&evaluate(interp, c)?),
            None => true,
        };
        if include {
            if let Err(e) = insert_fn(interp) {
                interp.env = original_env;
                return Err(e);
            }
        }
    }
    interp.env = original_env;
    Ok(())
}
