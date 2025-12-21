use crate::ast::Argument;
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::eval::functions::{call_value, evaluate_arg};
use crate::interpreter::eval::utils::to_iterable;
use crate::token::Span;

pub(crate) fn builtin_reduce(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<crate::ast::Value, EldritchError> {
    if args.len() < 2 || args.len() > 3 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "reduce() takes 2 or 3 arguments",
            span,
        );
    }
    let func_val = evaluate_arg(interp, &args[0])?;
    let iterable_val = evaluate_arg(interp, &args[1])?;
    let mut items = to_iterable(interp, &iterable_val, span)?.into_iter();

    let mut acc = if args.len() == 3 {
        evaluate_arg(interp, &args[2])?
    } else {
        match items.next() {
            Some(v) => v,
            None => {
                return interp.error(
                    EldritchErrorKind::TypeError,
                    "reduce() of empty sequence with no initial value",
                    span,
                );
            }
        }
    };

    for item in items {
        acc = call_value(interp, &func_val, &[acc, item], span)?;
    }
    Ok(acc)
}
