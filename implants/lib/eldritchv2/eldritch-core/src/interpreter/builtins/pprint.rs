use crate::ast::{Environment, Value};
use crate::token::Span;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;
use core::fmt::Write;

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

    let mut output = String::new();
    if pretty_format(&args[0], 0, indent_width, &mut output).is_err() {
        return Err("pprint(): failed to format output".to_string());
    }

    // TODO: Pass actual span
    env.borrow()
        .printer
        .print_out(&Span::new(0, 0, 0), &output);

    Ok(Value::None)
}

fn pretty_format(
    val: &Value,
    current_indent: usize,
    indent_width: usize,
    buf: &mut String,
) -> core::fmt::Result {
    match val {
        Value::List(l) => {
            let list = l.borrow();
            if list.is_empty() {
                return write!(buf, "[]");
            }
            write!(buf, "[\n")?;

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in list.iter().enumerate() {
                write!(buf, "{}", next_indent_str)?;
                pretty_format(item, next_indent, indent_width, buf)?;
                if i < list.len() - 1 {
                    write!(buf, ",")?;
                }
                write!(buf, "\n")?;
            }
            let indent_str = " ".repeat(current_indent);
            write!(buf, "{}]", indent_str)
        }
        Value::Dictionary(d) => {
            let dict = d.borrow();
            if dict.is_empty() {
                return write!(buf, "{{}}");
            }
            write!(buf, "{{\n")?;

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, (key, value)) in dict.iter().enumerate() {
                write!(buf, "{}{:?}: ", next_indent_str, key)?;
                pretty_format(value, next_indent, indent_width, buf)?;
                if i < dict.len() - 1 {
                    write!(buf, ",")?;
                }
                write!(buf, "\n")?;
            }
            let indent_str = " ".repeat(current_indent);
            write!(buf, "{}}}", indent_str)
        }
        Value::Tuple(t) => {
            if t.is_empty() {
                return write!(buf, "()");
            }
            write!(buf, "(\n")?;

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in t.iter().enumerate() {
                write!(buf, "{}", next_indent_str)?;
                pretty_format(item, next_indent, indent_width, buf)?;
                if i < t.len() - 1 || t.len() == 1 {
                    write!(buf, ",")?;
                }
                write!(buf, "\n")?;
            }
            let indent_str = " ".repeat(current_indent);
            write!(buf, "{})", indent_str)
        }
        Value::Set(s_val) => {
            let set = s_val.borrow();
            if set.is_empty() {
                return write!(buf, "set()"); // Python style
            }
            write!(buf, "{{\n")?;

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in set.iter().enumerate() {
                write!(buf, "{}", next_indent_str)?;
                pretty_format(item, next_indent, indent_width, buf)?;
                if i < set.len() - 1 {
                    write!(buf, ",")?;
                }
                write!(buf, "\n")?;
            }
            let indent_str = " ".repeat(current_indent);
            write!(buf, "{}}}", indent_str)
        }
        Value::String(s) => write!(buf, "{s:?}"),
        _ => write!(buf, "{val}"),
    }
}
