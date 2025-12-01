use super::get_all_builtins;
use crate::lang::ast::{Environment, Value};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn builtin_builtins(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("builtins() takes no arguments".to_string());
    }
    let mut names: Vec<String> = get_all_builtins()
        .into_iter()
        .map(|(n, _)| n.to_string())
        .collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_list))))
}
