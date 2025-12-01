use crate::lang::ast::{Environment, Value};
use crate::lang::global_libs::get_global_libraries;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn builtin_libs(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("libs() takes no arguments".to_string());
    }
    let libs = get_global_libraries();
    let mut names: Vec<String> = libs.keys().cloned().collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_list))))
}
