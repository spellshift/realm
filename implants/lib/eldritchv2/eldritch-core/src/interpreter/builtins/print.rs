use crate::ast::{Environment, Value};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;

pub fn builtin_print(env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    let mut out = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&arg.to_string());
    }

    env.borrow().printer.print(&out);
    Ok(Value::None)
}
