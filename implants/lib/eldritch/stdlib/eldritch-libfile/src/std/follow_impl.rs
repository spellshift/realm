#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::sync::Arc;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use eldritch_core::Printer;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn follow(
    path: String,
    fn_val: Value,
    printer: Arc<dyn Printer + Send + Sync>,
) -> Result<(), String> {
    follow_impl(path, fn_val, printer).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn follow(
    _path: alloc::string::String,
    _fn_val: eldritch_core::Value,
    _printer: alloc::sync::Arc<dyn eldritch_core::Printer + Send + Sync>,
) -> Result<(), alloc::string::String> {
    Err("follow requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn follow_impl(
    path: String,
    fn_val: Value,
    printer: Arc<dyn Printer + Send + Sync>,
) -> AnyhowResult<()> {
    use eldritch_core::Interpreter;
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use std::path::Path;

    // get pos to end of file
    let mut file = File::open(&path)?;
    let mut pos = fs::metadata(&path)?.len();

    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)?;

    // We need an interpreter to run the callback.
    // Use the printer passed from the calling interpreter so that native functions
    // like `print` output to the correct destination (e.g., server) instead of stdout.
    let mut interp = Interpreter::new_with_printer(printer);

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
    use alloc::sync::Arc;
    use eldritch_core::BufferPrinter;
    use eldritch_core::Interpreter;
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        let printer = interp.env.read().printer.clone();
        let res = follow(path, fn_val, printer);

        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("STOP"));
    }

    #[test]
    fn test_follow_with_native_print_directly() {
        // This test verifies that when the native `print` function is passed DIRECTLY
        // as the callback (not wrapped in a user function), the output goes to the
        // interpreter's printer, not stdout.
        //
        // BUG: This demonstrates the bug where passing a NativeFunction directly
        // causes follow_impl to create an interpreter with StdoutPrinter because
        // NativeFunction has no closure to extract the printer from.
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Write initial content
        ::std::fs::write(&path, "initial\n").unwrap();

        // Create a BufferPrinter to capture output
        let buffer_printer = Arc::new(BufferPrinter::new());
        let mut interp = Interpreter::new_with_printer(buffer_printer.clone());

        // Get the native `print` function directly
        let print_fn = interp.interpret("print").unwrap();

        // Verify it's a NativeFunction (not a user-defined Function)
        assert!(
            matches!(print_fn, Value::NativeFunction(_, _)),
            "Expected NativeFunction, got {:?}",
            print_fn
        );

        // Create a thread to append lines to the file
        let path_clone = path.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let mut file = OpenOptions::new().append(true).open(&path_clone).unwrap();
            // Write a line that print should output
            file.write_all(b"hello from follow\n").unwrap();
            file.flush().unwrap();
        });

        // Since we can't easily stop follow when using print directly (it won't error),
        // we need a different approach. We'll use a timeout.
        // For this test, we'll run follow in a background thread and check the buffer after.
        let path_for_follow = path.clone();
        let print_fn_clone = print_fn.clone();

        // Get the printer to pass to follow
        let printer_for_follow = buffer_printer.clone();

        let follow_handle = std::thread::spawn(move || {
            // This will block until an error or indefinitely
            let _ = follow(path_for_follow, print_fn_clone, printer_for_follow);
        });

        // Wait for the writer thread to complete
        handle.join().unwrap();

        // Give follow some time to process the new line
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check if output was captured in our buffer
        // BUG: This assertion will fail because follow_impl creates a new interpreter
        // with StdoutPrinter when the callback is a NativeFunction (no closure to extract printer from)
        let output = buffer_printer.read_out();

        // Clean up: we can't gracefully stop follow, so we'll just check the assertion
        // The follow_handle will be abandoned (thread will be terminated when test ends)
        drop(follow_handle);

        assert!(
            output.contains("hello from follow"),
            "Expected 'hello from follow' in buffer output, got: '{}'\n\
             This indicates the bug: when a NativeFunction is passed directly to file.follow,\n\
             the output goes to stdout instead of the interpreter's configured printer.",
            output
        );
    }
}
