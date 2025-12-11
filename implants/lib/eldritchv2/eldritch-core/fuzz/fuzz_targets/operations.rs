#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::{Unstructured, Result};
use eldritch_core::{Interpreter, Value, Printer, Span};
use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct NoOpPrinter;

impl Printer for NoOpPrinter {
    fn print_out(&self, _span: &Span, _s: &str) {}
    fn print_err(&self, _span: &Span, _s: &str) {}
}

// Helper to generate arbitrary Values.
// We limit depth to avoid stack overflow.
fn arbitrary_value(u: &mut Unstructured, depth: usize) -> Result<Value> {
    if depth == 0 {
        // Base cases
        let choice = u.int_in_range(0..=5)?;
        match choice {
            0 => Ok(Value::None),
            1 => Ok(Value::Bool(u.arbitrary()?)),
            2 => Ok(Value::Int(u.arbitrary()?)),
            3 => Ok(Value::Float(u.arbitrary()?)),
            4 => Ok(Value::String(u.arbitrary()?)),
            5 => Ok(Value::Bytes(u.arbitrary()?)),
            _ => unreachable!(),
        }
    } else {
        // Recursive cases
        let choice = u.int_in_range(0..=9)?;
        match choice {
            0 => Ok(Value::None),
            1 => Ok(Value::Bool(u.arbitrary()?)),
            2 => Ok(Value::Int(u.arbitrary()?)),
            3 => Ok(Value::Float(u.arbitrary()?)),
            4 => Ok(Value::String(u.arbitrary()?)),
            5 => Ok(Value::Bytes(u.arbitrary()?)),
            6 => {
                let len = u.int_in_range(0..=5)?;
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push(arbitrary_value(u, depth - 1)?);
                }
                Ok(Value::List(Arc::new(spin::RwLock::new(vec))))
            },
            7 => {
                let len = u.int_in_range(0..=5)?;
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push(arbitrary_value(u, depth - 1)?);
                }
                Ok(Value::Tuple(vec))
            },
            8 => {
                let len = u.int_in_range(0..=5)?;
                let mut map = BTreeMap::new();
                for _ in 0..len {
                    let k = arbitrary_value(u, depth - 1)?;
                    let v = arbitrary_value(u, depth - 1)?;
                    map.insert(k, v);
                }
                Ok(Value::Dictionary(Arc::new(spin::RwLock::new(map))))
            },
            9 => {
                let len = u.int_in_range(0..=5)?;
                let mut set = BTreeSet::new();
                for _ in 0..len {
                    set.insert(arbitrary_value(u, depth - 1)?);
                }
                Ok(Value::Set(Arc::new(spin::RwLock::new(set))))
            },
            _ => unreachable!(),
        }
    }
}

fn arbitrary_binary_operator(u: &mut Unstructured) -> Result<&'static str> {
    let ops = [
        "+", "-", "*", "/", "//", "%",
        "&", "|", "^", "<<", ">>",
        "==", "!=", "<", ">", "<=", ">=",
        "and", "or", "in", "not in"
    ];
    let idx = u.int_in_range(0..=ops.len() - 1)?;
    Ok(ops[idx])
}

fn arbitrary_unary_operator(u: &mut Unstructured) -> Result<&'static str> {
    let ops = ["not", "-", "+", "~"];
    let idx = u.int_in_range(0..=ops.len() - 1)?;
    Ok(ops[idx])
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let mut interpreter = Interpreter::new_with_printer(Arc::new(NoOpPrinter));

    // Determine whether to fuzz a unary or binary operation.
    // Let's say 2/3 chance for binary, 1/3 for unary.
    let is_binary = u.ratio(2, 3).unwrap_or(true);

    if is_binary {
        let val_a = match arbitrary_value(&mut u, 3) {
            Ok(v) => v,
            Err(_) => return,
        };
        let val_b = match arbitrary_value(&mut u, 3) {
            Ok(v) => v,
            Err(_) => return,
        };
        let op = match arbitrary_binary_operator(&mut u) {
            Ok(o) => o,
            Err(_) => return,
        };

        interpreter.define_variable("a", val_a);
        interpreter.define_variable("b", val_b);

        let code = format!("a {} b", op);
        let _ = interpreter.interpret(&code);
    } else {
        let val_a = match arbitrary_value(&mut u, 3) {
            Ok(v) => v,
            Err(_) => return,
        };
        let op = match arbitrary_unary_operator(&mut u) {
            Ok(o) => o,
            Err(_) => return,
        };

        interpreter.define_variable("a", val_a);

        // For unary ops, standard syntax is "op a", but for some (like post-fix) it might be different.
        // Eldritch only has prefix unary ops in the list above.
        let code = format!("{} a", op);
        let _ = interpreter.interpret(&code);
    }
});
