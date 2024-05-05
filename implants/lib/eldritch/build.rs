use anyhow::Result;



#[cfg(all(target_os = "windows", debug_assertions))]
fn build_bin_create_file_dll() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    // Define which files should cause this section to be rebuilt.
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\create_file_dll\\src\\lib.rs");
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\create_file_dll\\src\\main.rs");
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\create_file_dll\\Cargo.toml");

    // Get the path of the create_file_dll workspace member
    let cargo_root = env!("CARGO_MANIFEST_DIR");
    let relative_path_to_test_dll = "..\\..\\..\\bin\\create_file_dll\\";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll);
    println!("test_dll_path: {}", test_dll_path.to_str().unwrap());
    assert!(test_dll_path.is_dir());

    println!("Starting cargo build lib");
    let res = Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(test_dll_path)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .stderr
        .unwrap();

    let reader = BufReader::new(res);
    reader
        .lines()
        .map_while(Result::ok)
        .for_each(|line| println!("cargo dll build: {}", line));

    let relative_path_to_test_dll_file =
        "..\\..\\..\\bin\\create_file_dll\\target\\debug\\create_file_dll.dll";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(test_dll_path.is_file());
}

#[cfg(target_os = "windows")]
fn build_bin_reflective_loader() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    // Define which files should cause this section to be rebuilt.
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\reflective_loader\\src\\lib.rs");
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\reflective_loader\\src\\loader.rs");
    println!("cargo:rerun-if-changed=..\\..\\..\\bin\\reflective_loader\\Cargo.toml");

    // Get the path of the create_file_dll workspace member
    let cargo_root = env!("CARGO_MANIFEST_DIR");
    let relative_path_to_test_dll = "..\\..\\..\\bin\\reflective_loader\\";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll);
    assert!(test_dll_path.is_dir());

    println!("Starting cargo build lib");
    let res_build = Command::new("cargo")
        .args([
            "build",
            "--release",
            "-Z",
            "build-std=core,compiler_builtins",
            "-Z",
            "build-std-features=compiler-builtins-mem",
        ])
        .current_dir(test_dll_path.clone())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .stderr
        .unwrap();

    let reader = BufReader::new(res_build);
    reader
        .lines()
        .map_while(Result::ok)
        .for_each(|line| println!("cargo dll build: {}", line));

    let relative_path_to_test_dll_file = "..\\..\\..\\bin\\reflective_loader\\target\\x86_64-pc-windows-msvc\\release\\reflective_loader.dll";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(test_dll_path.is_file());
}

#[cfg(windows)]
const HOST_FAMILY: &str = "windows";

#[cfg(unix)]
const HOST_FAMILY: &str = "unix";

fn set_host_family() {
    println!("cargo:rustc-cfg=host_family=\"{}\"", HOST_FAMILY);
}

fn main() -> Result<()> {
    set_host_family();
    #[cfg(all(target_os = "windows", debug_assertions))]
    build_bin_create_file_dll();
    #[cfg(target_os = "windows")]
    build_bin_reflective_loader();

    Ok(())
}
