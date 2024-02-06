use std::path::Path;

use anyhow::{anyhow, Result};
use starlark::{eval::Evaluator, values::Value};

pub fn follow<'v>(path_str: String, f: Value<'v>, eval: &mut Evaluator<'v, '_>) -> Result<()> {
    let starlark_heap = eval.heap();
    let path = Path::new(&path_str);
    if path.is_dir() {
        for entry in path.read_dir()? {
            let entry = entry?;
            let entry_path = entry.path().to_str().ok_or(anyhow!("Failed to convert path to str"))?.to_string();
            follow(entry_path.to_string(), f, eval)?;
        }
    } else {
        let val = starlark_heap.alloc(path_str);
        eval.eval_function(f, &[val], &[])?;
    }
    Ok(())
}