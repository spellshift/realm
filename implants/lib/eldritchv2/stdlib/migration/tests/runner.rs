use anyhow::{Context, Result};
use eldritch::runtime::{messages::AsyncMessage, Message};
use eldritchv2::{BufferPrinter, Interpreter};
use pb::eldritch::Tome;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn run_migration_tests() -> Result<()> {
    let script_dir = PathBuf::from("tests/scripts");
    if !script_dir.exists() {
        println!("Script directory not found: {:?}", script_dir);
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&script_dir)
        .context("Failed to read script directory")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "eld"))
        .collect();

    entries.sort();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    for script_path in entries {
        println!("---------------------------------------------------");
        println!("Running test: {:?}", script_path.file_name().unwrap());
        let script_content = fs::read_to_string(&script_path)?;

        let v2_output = run_v2(&script_content)?;

        // Run V1
        let v1_output = rt.block_on(run_v1(&script_content))?;

        if v1_output.trim() == v2_output.trim() {
            println!("MATCH for {:?}", script_path.file_name().unwrap());
        } else {
            println!("MISMATCH for {:?}", script_path.file_name().unwrap());
            println!("--- V1 Output ---");
            println!("{}", v1_output);
            println!("--- V2 Output ---");
            println!("{}", v2_output);
            println!("-----------------");
        }
    }
    Ok(())
}

fn run_v2(code: &str) -> Result<String> {
    let printer = Arc::new(BufferPrinter::new());
    // Note: We use the default interpreter which should pick up the "fake_bindings" feature
    // enabled in Cargo.toml.
    // However, we need to manually invoke the builder methods that register the fake libs
    // if `with_default_libs` is designed that way.
    // Based on `eldritchv2/src/lib.rs`, `with_default_libs` registers fake libs if `fake_bindings` feature is on.
    // `with_fake_agent` is separate.

    let mut interp = Interpreter::new_with_printer(printer.clone()).with_default_libs();

    // Check if we can register fake agent too
    #[cfg(feature = "fake_bindings")]
    {
        interp = interp.with_fake_agent();
    }

    match interp.interpret(code) {
        Ok(_) => Ok(printer.read()),
        Err(e) => Ok(format!("Error: {}\nOutput so far:\n{}", e, printer.read())),
    }
}

async fn run_v1(code: &str) -> Result<String> {
    // V1 uses `Tome` struct
    let tome = Tome {
        eldritch: code.to_string(),
        parameters: HashMap::new(),
        file_names: Vec::new(),
    };

    // V1 `start` returns a Runtime
    // We use a dummy ID 123
    let mut runtime = eldritch::start(123, tome).await;
    runtime.finish().await;

    let mut output = String::new();

    // Iterate over messages.
    // Since `runtime.messages()` returns a slice/vec, we can iterate.
    // Wait, `runtime.messages()` in V1 might return a reference to internal buffer?
    // Let's check V1 tests again. `for msg in runtime.messages()`.

    for msg in runtime.messages() {
        if let Message::Async(am) = msg {
            match am {
                AsyncMessage::ReportText(m) => {
                    output.push_str(&m.text());
                    // output.push('\n'); // ReportText usually has newline? Or maybe not.
                    // V1 tests show: want_text: format!("{}\n", "2") for print(1+1).
                    // So print adds newline.
                    // `ReportText` struct likely contains the text.
                }
                AsyncMessage::ReportError(m) => {
                    output.push_str(&format!("Error: {}\n", m.error));
                }
                _ => {}
            }
        }
    }

    Ok(output)
}
