use crate::ast::{Argument, Value};
use crate::interpreter::core::{Flow, Interpreter};
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::eval::functions::evaluate_arg;
use crate::interpreter::eval::MAX_RECURSION_DEPTH;
use crate::token::Span;
use alloc::string::ToString;

pub(crate) fn builtin_eval_func(
    interp: &mut Interpreter,
    args: &[Argument],
    span: Span,
) -> Result<Value, EldritchError> {
    if args.len() != 1 {
        return interp.error(
            EldritchErrorKind::TypeError,
            "eval() takes exactly 1 argument",
            span,
        );
    }

    let code_val = evaluate_arg(interp, &args[0])?;
    let code = match code_val {
        Value::String(s) => s,
        _ => {
            return interp.error(
                EldritchErrorKind::TypeError,
                "eval() argument must be a string",
                span,
            );
        }
    };

    if interp.depth >= MAX_RECURSION_DEPTH {
        return interp.error(
            EldritchErrorKind::RecursionError,
            "Recursion limit exceeded",
            span,
        );
    }

    // Create a new interpreter instance that shares the environment
    // We manually construct it to avoid re-loading builtins and to set depth
    let mut temp_interp = Interpreter {
        env: interp.env.clone(),
        flow: Flow::Next,
        depth: interp.depth + 1,
        call_stack: interp.call_stack.clone(),
        current_func_name: "<eval>".to_string(),
        is_scope_owner: false,
    };

    match temp_interp.interpret(&code) {
        Ok(v) => Ok(v),
        Err(e) => interp.error(EldritchErrorKind::RuntimeError, &e, span),
    }
}
