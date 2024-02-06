use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};

use anyhow::{anyhow, Result};
use starlark::{eval::Evaluator, values::Value};

pub fn follow<'v>(path: String, f: Value<'v>, eval: &mut Evaluator<'v, '_>) -> Result<()> {
    let starlark_heap = eval.heap();
    // get pos to end of file
    let mut file = std::fs::File::open(&path)?;
    let mut pos = std::fs::metadata(&path)?.len();

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // watch
    for res in rx {
        match res {
            Ok(_event) => {
                // ignore any event that didn't change the pos
                if file.metadata()?.len() == pos {
                    continue;
                }

                // read from pos to end of file
                file.seek(std::io::SeekFrom::Start(pos))?;

                // update post to end of file
                pos = file.metadata()?.len();

                let reader = BufReader::new(&file);
                for line in reader.lines() {
                    let val = starlark_heap.alloc(line?.to_string());
                    eval.eval_function(f, &[val], &[])?;
                }
            }
            Err(_) => continue,
        }
    }
    Ok(())
}