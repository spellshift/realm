use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_dir_attributes;
use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn builtin_dir(env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
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

        let val_attrs: Vec<Value> = symbols.into_iter().map(Value::String).collect();
        return Ok(Value::List(Rc::new(RefCell::new(val_attrs))));
    }

    // Original behavior for dir(obj)
    let attrs = get_dir_attributes(&args[0]);
    let val_attrs: Vec<Value> = attrs.into_iter().map(Value::String).collect();
    Ok(Value::List(Rc::new(RefCell::new(val_attrs))))
}
