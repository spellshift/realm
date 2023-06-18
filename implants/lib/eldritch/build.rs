#[cfg(target_os = "windows")]
fn build_tests_create_file_dll(){
    use std::{process::{Command, Stdio}, path::Path, io::{BufReader, BufRead}};

    // Define which files should cause this section to be rebuilt.
    println!("cargo:rerun-if-changed=tests/create_file_dll/src/lib.rs");
    println!("cargo:rerun-if-changed=tests/create_file_dll/src/main.rs");
    println!("cargo:rerun-if-changed=tests/create_file_dll/Cargo.toml");

    // Get the path of the create_file_dll workspace member
    let cargo_root = env!("CARGO_MANIFEST_DIR");
    let relative_path_to_test_dll = "..\\..\\..\\tests\\create_file_dll\\";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll);
    assert!(test_dll_path.is_dir());

    println!("Starting cargo build lib");
    let res = Command::new("cargo").args(&["build","--lib"])
        .current_dir(test_dll_path)
        .stderr(Stdio::piped())
        .spawn().unwrap().stderr.unwrap();

    let reader = BufReader::new(res);
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("cargo dll build: {}", line));

    let relative_path_to_test_dll_file = "..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(test_dll_path.is_file());
}

fn main() {
    #[cfg(target_os = "windows")]
    build_tests_create_file_dll();
}
