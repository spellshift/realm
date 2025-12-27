#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn follow(path: String, fn_val: Value) -> Result<(), String> {
    follow_impl(path, fn_val).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn follow(_path: alloc::string::String, _fn_val: Value) -> Result<(), alloc::string::String> {
    Err("follow requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn follow_impl(path: String, fn_val: Value) -> AnyhowResult<()> {
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use std::path::Path;
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use eldritch_core::Interpreter;

    // get pos to end of file
    let mut file = File::open(&path)?;
    let mut pos = fs::metadata(&path)?.len();

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

    // We need an interpreter to run the callback.
    // If it's a user function, it captures its environment (closure).
    // If it's native (like print), it needs an environment with a printer.
    // We try to re-use the printer from the closure if available, else default.

    let mut printer = None;
    if let Value::Function(f) = &fn_val {
        printer = Some(f.closure.read().printer.clone());
    }

    // Since this is blocking, we can create one interpreter instance and reuse it
    let mut interp = if let Some(p) = printer {
        Interpreter::new_with_printer(p)
    } else {
        Interpreter::new()
    };

    // watch
    for _event in rx.into_iter().flatten() {
        // ignore any event that didn't change the pos
        if let Ok(meta) = file.metadata() {
            if meta.len() == pos {
                continue;
            }
        } else {
            continue;
        }

        // read from pos to end of file
        file.seek(SeekFrom::Start(pos))?;

        let mut reader = BufReader::new(&file);
        let mut bytes_read = 0;

        loop {
            let mut line = String::new();
            // read_line includes the delimiter
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            bytes_read += n as u64;

            // Trim trailing newline for consistency with lines() which strips it?
            // V1 used `reader.lines()` which strips newline.
            // read_line keeps it. We should strip it.
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            let line_val = Value::String(line);

            // Execute callback
            // We use define_variable + interpret as a robust way to call without internal API access
            interp.define_variable("_follow_cb", fn_val.clone());
            interp.define_variable("_follow_line", line_val);

            match interp.interpret("_follow_cb(_follow_line)") {
                Ok(_) => {}
                Err(e) => return Err(anyhow::anyhow!(e)),
            }
        }

        // update pos based on actual bytes read
        pos += bytes_read;
    }
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::fs::OpenOptions;
    use eldritch_core::Interpreter;

    #[test]
    fn test_follow() {
        // We verify that follow can be called and executes callback.
        // Since it's blocking, we use a callback that throws an error to exit the loop.
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Write initial content
        ::std::fs::write(&path, "line1\n").unwrap();

        // Create a thread to update file after a delay, to trigger watcher
        let path_clone = path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            // Append line
            let mut file = OpenOptions::new().append(true).open(path_clone).unwrap();
            file.write_all(b"line2\n").unwrap();
        });

        // Define a native function value that simulates a callback throwing an error on specific input
        // Since we can't easily construct a Value::Function here without parsing (unless we use Interpreter to make one),
        // we can use Interpreter to create the value.

        let mut interp = Interpreter::new();
        let code = r#"
def cb(line):
    if line == "line2":
        fail("STOP")
cb
"#;
        let fn_val = interp.interpret(code).map_err(|e| e).unwrap();

        // Call follow. It should block until "line2" is written, then cb is called, throws error, and follow returns Err.
        let res = follow(path, fn_val);

        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("STOP"));
    }
}
