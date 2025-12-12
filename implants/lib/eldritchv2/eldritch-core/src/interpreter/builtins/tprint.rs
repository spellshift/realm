use crate::ast::{Environment, Value};
use crate::token::Span;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::{BTreeSet, BTreeMap};
use alloc::sync::Arc;
use spin::RwLock;

/// `tprint(list_of_dicts)`: Prints a list of dictionaries as a markdown table.
pub fn builtin_tprint(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("tprint() takes at least 1 argument".to_string());
    }

    let list_val = &args[0];
    let items_snapshot: Vec<Value> = match list_val {
        Value::List(l) => l.read().clone(),
        _ => return Err("tprint() argument must be a list of dictionaries".to_string()),
    };

    if items_snapshot.is_empty() {
        return Ok(Value::None);
    }

    // Collect all unique keys (columns)
    let mut columns: BTreeSet<String> = BTreeSet::new();
    let mut rows: Vec<BTreeMap<String, String>> = Vec::new();

    for item in items_snapshot {
        match item {
            Value::Dictionary(d) => {
                // Snapshot the dictionary content to release the lock immediately
                let dict_snapshot: BTreeMap<Value, Value> = d.read().clone();
                let mut row_map: BTreeMap<String, String> = BTreeMap::new();
                for (k, v) in dict_snapshot {
                    let key_str = k.to_string();
                    columns.insert(key_str.clone());

                    let val_str = v.to_string()
                        .replace('\n', "\\n")
                        .replace('|', "\\|");
                    row_map.insert(key_str, val_str);
                }
                rows.push(row_map);
            }
            _ => return Err("tprint() list must contain only dictionaries".to_string()),
        }
    }

    if columns.is_empty() {
         return Ok(Value::None);
    }

    let columns_vec: Vec<String> = columns.into_iter().collect();

    // Calculate widths
    let mut widths: BTreeMap<String, usize> = BTreeMap::new();
    for col in &columns_vec {
        widths.insert(col.clone(), col.len());
    }

    for row in &rows {
        for col in &columns_vec {
            let val = row.get(col).map(|s| s.as_str()).unwrap_or("");
            let w = widths.get_mut(col).unwrap();
            *w = (*w).max(val.len());
        }
    }

    let mut output = String::new();

    // Print Header
    output.push('|');
    for col in &columns_vec {
        let w = widths.get(col).unwrap();
        output.push_str(&format!(" {:width$} |", col, width = w));
    }
    output.push('\n');

    // Print Separator
    output.push('|');
    for col in &columns_vec {
        let w = widths.get(col).unwrap();
        let dash_count = *w;
        output.push(' ');
        output.push_str(&"-".repeat(dash_count));
        output.push_str(" |");
    }
    output.push('\n');

    // Print Rows
    for row in &rows {
        output.push('|');
        for col in &columns_vec {
            let w = widths.get(col).unwrap();
            let val = row.get(col).map(|s| s.as_str()).unwrap_or("");
            output.push_str(&format!(" {:width$} |", val, width = w));
        }
        output.push('\n');
    }

    env.read().printer.print_out(&Span::new(0, 0, 0), &output);

    Ok(Value::None)
}
