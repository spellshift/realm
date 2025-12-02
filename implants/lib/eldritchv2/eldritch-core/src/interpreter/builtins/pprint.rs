use crate::ast::{Environment, Value};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;

pub fn builtin_pprint(env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    let indent_width = if args.len() > 1 {
        match args[1] {
            Value::Int(i) => i.max(0) as usize,
            _ => return Err("pprint() indent must be an integer".to_string()),
        }
    } else {
        2 // Default indent
    };

    if args.is_empty() {
        return Err("pprint() takes at least 1 argument".to_string());
    }

    let output = pretty_format(&args[0], 0, indent_width);
    env.borrow().printer.print(&output);

    Ok(Value::None)
}

fn pretty_format(val: &Value, current_indent: usize, indent_width: usize) -> String {
    let indent_str = " ".repeat(current_indent);

    match val {
        Value::List(l) => {
            let list = l.borrow();
            if list.is_empty() {
                return "[]".to_string();
            }
            let mut s = "[\n".to_string();
            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in list.iter().enumerate() {
                s.push_str(&format!(
                    "{}{}",
                    next_indent_str,
                    pretty_format(item, next_indent, indent_width)
                ));
                if i < list.len() - 1 {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str(&format!("{indent_str}]"));
            s
        }
        Value::Dictionary(d) => {
            let dict = d.borrow();
            if dict.is_empty() {
                return "{}".to_string();
            }
            let mut s = "{\n".to_string();
            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, (key, value)) in dict.iter().enumerate() {
                s.push_str(&format!("{next_indent_str}: {key}"));
                s.push_str(": ");
                s.push_str(&pretty_format(value, next_indent, indent_width));
                if i < dict.len() - 1 {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str(&format!("{indent_str}}}"));
            s
        }
        Value::Tuple(t) => {
            if t.is_empty() {
                return "()".to_string();
            }
            let mut s = "(\n".to_string();
            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in t.iter().enumerate() {
                s.push_str(&format!(
                    "{}{}",
                    next_indent_str,
                    pretty_format(item, next_indent, indent_width)
                ));
                if i < t.len() - 1 || t.len() == 1 {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str(&format!("{indent_str})"));
            s
        }
        Value::Set(s_val) => {
            let set = s_val.borrow();
            if set.is_empty() {
                return "set()".to_string(); // Python style
            }
            let mut s = "{\n".to_string();
            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in set.iter().enumerate() {
                s.push_str(&format!(
                    "{}{}",
                    next_indent_str,
                    pretty_format(item, next_indent, indent_width)
                ));
                if i < set.len() - 1 {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str(&format!("{indent_str}}}"));
            s
        }
        _ => format!("{val}"),
    }
}
