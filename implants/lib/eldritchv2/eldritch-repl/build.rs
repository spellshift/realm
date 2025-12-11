use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // If we are already building for WASM, do not recurse.
    let target = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target == "wasm32" {
        return;
    }

    println!("cargo:rerun-if-changed=src/wasm.rs");
    println!("cargo:rerun-if-changed=www/index.html");
    println!("cargo:rerun-if-changed=build.rs");

    // Check if wasm-pack is installed
    if Command::new("wasm-pack")
        .arg("--version")
        .output()
        .is_err()
    {
        println!("cargo:warning=wasm-pack not found. Skipping WASM REPL build.");
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest_dir);

    // Run wasm-pack build
    // wasm-pack build --target web --out-dir www/pkg --no-typescript -- --features fake_bindings
    let status = Command::new("wasm-pack")
        .current_dir(root)
        .args(&[
            "build",
            "--target",
            "web",
            "--out-dir",
            "www/pkg",
            "--no-typescript",
            "--",
            "--features",
            "fake_bindings",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            // Build successful
        }
        _ => {
            println!("cargo:warning=wasm-pack build failed. Skipping deployment.");
            return;
        }
    }

    // Deploy to docs
    // Target: docs/assets/eldritch-repl (relative to repo root)
    // From current dir (implants/lib/eldritchv2/eldritch-repl) -> ../../../../docs/assets/eldritch-repl
    let docs_dir = root.join("../../../../docs/assets/eldritch-repl");

    if let Err(e) = fs::create_dir_all(&docs_dir) {
        println!("cargo:warning=Failed to create docs directory: {}", e);
        return;
    }

    // Copy index.html
    let src_index = root.join("www/index.html");
    if let Err(e) = fs::copy(&src_index, docs_dir.join("index.html")) {
        println!("cargo:warning=Failed to copy index.html: {}", e);
    }

    // Copy pkg directory
    let src_pkg = root.join("www/pkg");
    let dest_pkg = docs_dir.join("pkg");
    if let Err(e) = fs::create_dir_all(&dest_pkg) {
        println!("cargo:warning=Failed to create pkg directory: {}", e);
    }

    if let Ok(entries) = fs::read_dir(&src_pkg) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    let _ = fs::copy(&path, dest_pkg.join(name));
                }
            }
        }
    }
}
