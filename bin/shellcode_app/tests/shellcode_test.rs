use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test that builds shellcode and verifies it's position-independent
/// and executes correctly as a binary
#[test]
fn test_shellcode_properties() {
    // Build the shellcode with the shellcode feature
    let build_output = Command::new("bash")
        .arg("build.sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run build.sh");

    assert!(
        build_output.status.success(),
        "Failed to build shellcode: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Extract the shellcode
    let shellcode_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/x86_64-unknown-linux-gnu/release/shellcode_app");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let shellcode_bin = temp_dir.path().join("shellcode.bin");

    let objcopy_output = Command::new("objcopy")
        .args(&[
            "-O",
            "binary",
            "--only-section=.text",
            shellcode_path.to_str().unwrap(),
            shellcode_bin.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run objcopy");

    assert!(
        objcopy_output.status.success(),
        "Failed to extract shellcode"
    );

    // Read the shellcode bytes
    let shellcode = fs::read(&shellcode_bin).expect("Failed to read shellcode");
    println!("Shellcode size: {} bytes", shellcode.len());

    // Verify it's position-independent by checking for relocations
    let readelf_output = Command::new("readelf")
        .args(&["-r", shellcode_path.to_str().unwrap()])
        .output()
        .expect("Failed to run readelf");

    let readelf_str = String::from_utf8_lossy(&readelf_output.stdout);
    assert!(
        readelf_str.contains("no relocations"),
        "Shellcode contains relocations - not position-independent!"
    );

    println!("✓ Shellcode is position-independent (no relocations)");

    // Test executing the shellcode binary
    let test_file = temp_dir.path().join("created_by_shellcode.txt");

    let output = Command::new(&shellcode_path)
        .env("SHELLCODE_FILE_PATH", test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute shellcode binary");

    assert!(
        output.status.success(),
        "Shellcode binary failed to execute"
    );

    assert!(
        test_file.exists(),
        "Shellcode binary did not create file at {:?}",
        test_file
    );

    println!("✓ Shellcode binary execution successful");
    println!("✓ All shellcode properties verified");
}

/// Test that the regular binary (non-shellcode mode) works correctly
#[test]
fn test_regular_binary_execution() {
    // Build without the shellcode feature
    let build_output = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to build");

    assert!(
        build_output.status.success(),
        "Failed to build: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_file.txt");

    // Run the binary
    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/x86_64-unknown-linux-gnu/release/shellcode_app");

    let output = Command::new(&binary_path)
        .env("SHELLCODE_FILE_PATH", test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Binary failed: exit code {}",
        output.status
    );

    assert!(
        test_file.exists(),
        "Binary did not create file at {:?}",
        test_file
    );

    println!("✓ Regular binary execution successful");
}
