use super::utils::{get_dir_attributes, get_type_name, is_truthy};
use crate::ast::{BuiltinFn, Value};
use crate::get_global_libraries;
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
    ]
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    #[cfg(feature = "std")]
    {
        println!("{}", args[0]);
    }
    #[cfg(not(feature = "std"))]
    let _ = args;
    Ok(Value::None)
}

fn builtin_len(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Dictionary(d) => Ok(Value::Int(d.borrow().len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        _ => Err(format!("'len()' is not defined for type: {:?}", args[0])),
    }
}

fn builtin_range(args: &[Value]) -> Result<Value, String> {
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

fn builtin_type(args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(get_type_name(&args[0])))
}

fn builtin_bool(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(is_truthy(&args[0])))
}

fn builtin_str(args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(args[0].to_string()))
}

fn builtin_int(args: &[Value]) -> Result<Value, String> {
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

fn builtin_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        // Return list of standard built-ins names
        let mut builtins = vec![
            "assert",
            "assert_eq",
            "bool",
            "dir",
            "enumerate",
            "fail",
            "filter",
            "int",
            "len",
            "map",
            "print",
            "range",
            "reduce",
            "str",
            "type",
        ];

        // Add registered libraries
        let libs = get_global_libraries();
        let mut lib_names: Vec<&str> = libs.keys().map(|s| s.as_str()).collect();
        builtins.append(&mut lib_names);
        builtins.sort();

        let val_attrs: Vec<Value> = builtins
            .into_iter()
            .map(|s| Value::String(s.to_string()))
            .collect();
        return Ok(Value::List(Rc::new(RefCell::new(val_attrs))));
    }
    let attrs = get_dir_attributes(&args[0]);
    let val_attrs: Vec<Value> = attrs.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_attrs))))
}

fn builtin_enumerate(args: &[Value]) -> Result<Value, String> {
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

fn builtin_assert(args: &[Value]) -> Result<Value, String> {
    if !is_truthy(&args[0]) {
        return Err(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        ));
    }
    Ok(Value::None)
}

fn builtin_assert_eq(args: &[Value]) -> Result<Value, String> {
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}

fn builtin_fail(args: &[Value]) -> Result<Value, String> {
    Err(format!("Test failed explicitly: {}", args[0]))
}

fn builtin_stub(_args: &[Value]) -> Result<Value, String> {
    Err("internal error: this function should be handled by interpreter".to_string())
}
