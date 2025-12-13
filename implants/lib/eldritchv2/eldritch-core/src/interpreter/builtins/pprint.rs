use crate::ast::{Environment, Value};
use crate::token::Span;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

/// `pprint(object, indent=2)`: Pretty-prints an object.
///
/// Prints the object in a formatted, readable way with indentation.
/// Useful for debugging complex data structures like dictionaries and lists.
pub fn builtin_pprint(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
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
    pretty_format(&args[0], 0, indent_width, &mut output);

    // TODO: Pass actual span
    env.read().printer.print_out(&Span::new(0, 0, 0), &output);

    Ok(Value::None)
}

fn pretty_format(val: &Value, current_indent: usize, indent_width: usize, buf: &mut String) {
    match val {
        Value::List(l) => {
            let list = l.read();
            if list.is_empty() {
                buf.push_str("[]");
                return;
            }
            buf.push_str("[\n");

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in list.iter().enumerate() {
                buf.push_str(&next_indent_str);
                pretty_format(item, next_indent, indent_width, buf);
                if i < list.len() - 1 {
                    buf.push(',');
                }
                buf.push('\n');
            }
            let indent_str = " ".repeat(current_indent);
            buf.push_str(&indent_str);
            buf.push(']');
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            if dict.is_empty() {
                buf.push_str("{}");
                return;
            }
            buf.push_str("{\n");

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, (key, value)) in dict.iter().enumerate() {
                buf.push_str(&next_indent_str);
                buf.push_str(&format!("{key:?}: "));
                pretty_format(value, next_indent, indent_width, buf);
                if i < dict.len() - 1 {
                    buf.push(',');
                }
                buf.push('\n');
            }
            let indent_str = " ".repeat(current_indent);
            buf.push_str(&indent_str);
            buf.push('}');
        }
        Value::Tuple(t) => {
            if t.is_empty() {
                buf.push_str("()");
                return;
            }
            buf.push_str("(\n");

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in t.iter().enumerate() {
                buf.push_str(&next_indent_str);
                pretty_format(item, next_indent, indent_width, buf);
                if i < t.len() - 1 || t.len() == 1 {
                    buf.push(',');
                }
                buf.push('\n');
            }
            let indent_str = " ".repeat(current_indent);
            buf.push_str(&indent_str);
            buf.push(')');
        }
        Value::Set(s_val) => {
            let set = s_val.read();
            if set.is_empty() {
                buf.push_str("set()");
                return;
            }
            buf.push_str("{\n");

            let next_indent = current_indent + indent_width;
            let next_indent_str = " ".repeat(next_indent);

            for (i, item) in set.iter().enumerate() {
                buf.push_str(&next_indent_str);
                pretty_format(item, next_indent, indent_width, buf);
                if i < set.len() - 1 {
                    buf.push(',');
                }
                buf.push('\n');
            }
            let indent_str = " ".repeat(current_indent);
            buf.push_str(&indent_str);
            buf.push('}');
        }
        Value::String(s) => buf.push_str(&format!("{s:?}")),
        _ => buf.push_str(&format!("{val}")),
    }
}
