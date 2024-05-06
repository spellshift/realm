use anyhow::Result;

#[cfg(debug_assertions)]
fn build_bin_create_file_dll() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    // Define which files should cause this section to be rebuilt.
    println!("cargo:rerun-if-changed=../../../bin/create_file_dll/src/lib.rs");
    println!("cargo:rerun-if-changed=../../../bin/create_file_dll/src/main.rs");
    println!("cargo:rerun-if-changed=../../../bin/create_file_dll/Cargo.toml");

    // Get the path of the create_file_dll workspace member
    let cargo_root = env!("CARGO_MANIFEST_DIR");
    let relative_path_to_test_dll = "../../../bin/create_file_dll/";
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
        "../../../bin/create_file_dll/target/debug/create_file_dll.dll";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(test_dll_path.is_file());
}

fn build_bin_reflective_loader() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    let target_arch = std::env::var_os("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_arch_str = target_arch.to_str().unwrap();
    let target_vendor = std::env::var_os("CARGO_CFG_TARGET_VENDOR").unwrap();
    let target_vendor_str = target_vendor.to_str().unwrap();
    let target_os = std::env::var_os("CARGO_CFG_TARGET_OS").unwrap();
    let target_os_str = target_os.to_str().unwrap();
    let target_env = std::env::var_os("CARGO_CFG_TARGET_ENV").unwrap();
    let target_env_str = target_env.to_str().unwrap();

    let target_triple =
        format!("{target_arch_str}-{target_vendor_str}-{target_os_str}-{target_env_str}");

    let cargo_root = env!("CARGO_MANIFEST_DIR");

    let reflective_loader_path_str = "../../../bin/reflective_loader";
    let loader_files = [
        "src/lib.rs",
        "src/loader.rs",
        "Cargo.toml",
        &format!("target/{target_triple}/release/reflective_loader.dll"),
    ];
    // Define which files should cause this section to be rebuilt.
    for f in loader_files {
        let binding = format!("{}/{}", reflective_loader_path_str, f);
        let tmp_path = Path::new(cargo_root).join(binding.as_str());
        let tmp_str = tmp_path.to_str().unwrap();
        println!("cargo:rerun-if-changed={tmp_str}");
    }

    // Get the path of the create_file_dll workspace member
    let relative_path_to_test_dll = "../../../bin/reflective_loader/";
    let test_dll_path = Path::new(cargo_root)
        .join(relative_path_to_test_dll)
        .canonicalize()
        .unwrap();
    assert!(test_dll_path.is_dir());

    println!("Starting cargo build lib");
    // Define custom builds based on the target triple
    let res_build = match target_triple.as_str() {
        "x86_64-pc-windows-msvc" => Command::new("cargo")
            .args([
                "build",
                "--release",
                "-Z",
                "build-std=core,compiler_builtins",
                "-Z",
                "build-std-features=compiler-builtins-mem",
                &format!("--target={target_triple}"),
            ])
            .current_dir(test_dll_path.clone())
            .env("RUSTFLAGS", "-C target-feature=+crt-static")
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .stderr
            .unwrap(),
        _ => Command::new("cargo")
            .args([
                "build",
                "--release",
                "--lib",
                &format!("--target={target_triple}"),
            ])
            .current_dir(test_dll_path.clone())
            .env("RUSTFLAGS", "-C target-feature=+crt-static")
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
            .stderr
            .unwrap(),
    };

    let reader = BufReader::new(res_build);
    reader
        .lines()
        .map_while(Result::ok)
        .for_each(|line| println!("cargo dll build: {}", line));

    let relative_path_to_test_dll_file = format!(
        "../../../bin/reflective_loader/target/{target_triple}/release/reflective_loader.dll"
    );
    let loader_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(loader_dll_path.is_file());
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

    let binding = std::env::var_os("CARGO_CFG_TARGET_OS").unwrap();
    let build_target_os = binding.to_str().unwrap();

    if build_target_os == "windows" {
        #[cfg(debug_assertions)]
        build_bin_create_file_dll();
        build_bin_reflective_loader();
    }
    Ok(())
}
