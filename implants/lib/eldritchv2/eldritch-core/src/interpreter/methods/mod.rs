use crate::ast::Value;
use crate::interpreter::introspection::{find_best_match, get_type_name};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

mod dict;
mod list;
mod set;
mod str;

use dict::handle_dict_methods;
use list::handle_list_methods;
use set::handle_set_methods;
use str::handle_string_methods;

// --- Argument Validation Helper ---

pub trait ArgCheck {
    fn require(&self, count: usize, name: &str) -> Result<(), String>;
    fn require_range(&self, min: usize, max: usize, name: &str) -> Result<(), String>;
}

impl ArgCheck for [Value] {
    fn require(&self, count: usize, name: &str) -> Result<(), String> {
        if self.len() != count {
            return Err(format!(
                "TypeError: {}() takes exactly {} argument{}",
                name,
                count,
                if count != 1 { "s" } else { "" }
            ));
        }
        Ok(())
    }

    fn require_range(&self, min: usize, max: usize, name: &str) -> Result<(), String> {
        if self.len() < min || self.len() > max {
            return Err(format!(
                "TypeError: {}() takes between {} and {} arguments",
                name, min, max
            ));
        }
        Ok(())
    }
}

pub fn get_native_methods(value: &Value) -> Vec<String> {
    match value {
        Value::List(_) => vec![
            "append".to_string(),
            "extend".to_string(),
            "insert".to_string(),
            "remove".to_string(),
            "index".to_string(),
            "pop".to_string(),
            "sort".to_string(),
        ],
        Value::Dictionary(_) => vec![
            "keys".to_string(),
            "values".to_string(),
            "items".to_string(),
            "get".to_string(),
            "update".to_string(),
            "popitem".to_string(),
        ],
        Value::Set(_) => vec![
            "add".to_string(),
            "clear".to_string(),
            "contains".to_string(),
            "difference".to_string(),
            "discard".to_string(),
            "intersection".to_string(),
            "isdisjoint".to_string(),
            "issubset".to_string(),
            "issuperset".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "symmetric_difference".to_string(),
            "union".to_string(),
            "update".to_string(),
        ],
        Value::String(_) => vec![
            "split".to_string(),
            "splitlines".to_string(),
            "strip".to_string(),
            "lstrip".to_string(),
            "rstrip".to_string(),
            "lower".to_string(),
            "upper".to_string(),
            "capitalize".to_string(),
            "title".to_string(),
            "startswith".to_string(),
            "endswith".to_string(),
            "removeprefix".to_string(),
            "removesuffix".to_string(),
            "find".to_string(),
            "index".to_string(),
            "rfind".to_string(),
            "rindex".to_string(),
            "count".to_string(),
            "replace".to_string(),
            "join".to_string(),
            "format".to_string(),
            "partition".to_string(),
            "rpartition".to_string(),
            "rsplit".to_string(),
            "codepoints".to_string(),
            "elems".to_string(),
            "isalnum".to_string(),
            "isalpha".to_string(),
            "isdigit".to_string(),
            "islower".to_string(),
            "isupper".to_string(),
            "isspace".to_string(),
            "istitle".to_string(),
        ],
        _ => Vec::new(),
    }
}

pub fn call_bound_method(receiver: &Value, method: &str, args: &[Value]) -> Result<Value, String> {
    let result = match receiver {
        Value::List(l) => handle_list_methods(l, method, args),
        Value::Dictionary(d) => handle_dict_methods(d, method, args),
        Value::Set(s) => handle_set_methods(s, method, args),
        Value::String(s) => handle_string_methods(s, method, args),
        _ => None,
    };

    match result {
        Some(res) => res,
        None => {
            let mut msg = format!(
                "Object of type '{}' has no method '{}'",
                get_type_name(receiver),
                method
            );
            // Suggest similar methods
            let candidates = get_native_methods(receiver);
            if let Some(suggestion) = find_best_match(method, &candidates) {
                msg.push_str(&format!("\nDid you mean '{suggestion}'?"));
            }
            Err(msg)
        }
    }
}
