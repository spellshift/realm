use super::utils::{get_dir_attributes, get_type_name, is_truthy};
use super::super::ast::{BuiltinFn, Environment, Value};
use crate::lang::global_libs::get_global_libraries;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn get_all_builtins() -> Vec<(&'static str, BuiltinFn)> {
    vec![
        ("print", builtin_print as BuiltinFn),
        ("len", builtin_len),
        ("range", builtin_range),
        ("type", builtin_type),
        ("bool", builtin_bool),
        ("str", builtin_str),
        ("int", builtin_int),
        ("dir", builtin_dir),
        ("assert", builtin_assert),
        ("assert_eq", builtin_assert_eq),
        ("fail", builtin_fail),
        ("enumerate", builtin_enumerate),
        ("map", builtin_stub),
        ("filter", builtin_stub),
        ("reduce", builtin_stub),
        ("libs", builtin_libs),
        ("builtins", builtin_builtins),
        ("bytes", builtin_bytes),
    ]
}

fn builtin_print(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    #[cfg(feature = "std")]
    {
        println!("{}", args[0]);
    }
    #[cfg(not(feature = "std"))]
    let _ = args;
    Ok(Value::None)
}

fn builtin_len(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Dictionary(d) => Ok(Value::Int(d.borrow().len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        _ => Err(format!("'len()' is not defined for type: {:?}", args[0])),
    }
}

fn builtin_range(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    let (start, end) = match args {
        [Value::Int(end)] => (0, *end),
        [Value::Int(start), Value::Int(end)] => (*start, *end),
        _ => return Err("Range expects one or two integer arguments.".to_string()),
    };
    let mut list = Vec::new();
    if start < end {
        for i in start..end {
            list.push(Value::Int(i));
        }
    }
    Ok(Value::List(Rc::new(RefCell::new(list))))
}

fn builtin_type(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(get_type_name(&args[0])))
}

fn builtin_bool(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(is_truthy(&args[0])))
}

fn builtin_str(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(args[0].to_string()))
}

fn builtin_int(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("invalid literal for int(): '{}'", s)),
        _ => Err(format!(
            "int() argument must be a string, bytes or number, not '{}'",
            get_type_name(&args[0])
        )),
    }
}

fn builtin_bytes(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("bytes() expects exactly one argument".to_string());
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Bytes(s.as_bytes().to_vec())),
        Value::List(l) => {
            let list = l.borrow();
            let mut bytes = Vec::with_capacity(list.len());
            for item in list.iter() {
                match item {
                    Value::Int(i) => {
                        if *i < 0 || *i > 255 {
                            return Err(format!("bytes() list items must be integers in range 0-255, got {}", i));
                        }
                        bytes.push(*i as u8);
                    }
                    _ => return Err(format!("bytes() list items must be integers, got {}", get_type_name(item))),
                }
            }
            Ok(Value::Bytes(bytes))
        }
        Value::Int(i) => {
            if *i < 0 {
                return Err("bytes() argument cannot be negative".to_string());
            }
            Ok(Value::Bytes(vec![0; *i as usize]))
        }
        _ => Err(format!(
            "bytes() argument must be a string, list of integers, or integer size, not '{}'",
            get_type_name(&args[0])
        )),
    }
}

fn builtin_dir(env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        let mut symbols = BTreeSet::new();
        let mut current_env = Some(Rc::clone(env));

        // Walk up the environment chain
        while let Some(env_rc) = current_env {
            let env_ref = env_rc.borrow();
            for key in env_ref.values.keys() {
                symbols.insert(key.clone());
            }
            current_env = env_ref.parent.clone();
        }

        let val_attrs: Vec<Value> = symbols
            .into_iter()
            .map(Value::String)
            .collect();
        return Ok(Value::List(Rc::new(RefCell::new(val_attrs))));
    }

    // Original behavior for dir(obj)
    let attrs = get_dir_attributes(&args[0]);
    let val_attrs: Vec<Value> = attrs.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_attrs))))
}

fn builtin_libs(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("libs() takes no arguments".to_string());
    }
    let libs = get_global_libraries();
    let mut names: Vec<String> = libs.keys().cloned().collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_list))))
}

fn builtin_builtins(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("builtins() takes no arguments".to_string());
    }
    let mut names: Vec<String> = get_all_builtins().into_iter().map(|(n, _)| n.to_string()).collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_list))))
}

fn builtin_enumerate(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    let iterable = &args[0];
    let start = if args.len() > 1 {
        match args[1] {
            Value::Int(i) => i,
            _ => return Err("enumerate() start must be an integer".to_string()),
        }
    } else {
        0
    };
    let items = match iterable {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        _ => {
            return Err(format!(
                "Type '{:?}' is not iterable",
                get_type_name(iterable)
            ))
        }
    };
    let mut pairs = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        pairs.push(Value::Tuple(vec![Value::Int(i as i64 + start), item]));
    }
    Ok(Value::List(Rc::new(RefCell::new(pairs))))
}

fn builtin_assert(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !is_truthy(&args[0]) {
        return Err(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        ));
    }
    Ok(Value::None)
}

fn builtin_assert_eq(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}

fn builtin_fail(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Err(format!("Test failed explicitly: {}", args[0]))
}

fn builtin_stub(_env: &Rc<RefCell<Environment>>, _args: &[Value]) -> Result<Value, String> {
    Err("internal error: this function should be handled by interpreter".to_string())
}
