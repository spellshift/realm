use anyhow::Result;

#[cfg(debug_assertions)]
fn build_bin_create_file_dll() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    // Define which files should cause this section to be rebuilt.
    println!("cargo:rerun-if-changed=../../../../../bin/create_file_dll/src/lib.rs");
    println!("cargo:rerun-if-changed=../../../../../bin/create_file_dll/src/main.rs");
    println!("cargo:rerun-if-changed=../../../../../bin/create_file_dll/Cargo.toml");

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

    // Get the path of the create_file_dll workspace member
    let cargo_root = env!("CARGO_MANIFEST_DIR");
    let relative_path_to_test_dll = "../../../../../bin/create_file_dll/";
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll);
    println!("test_dll_path: {}", test_dll_path.to_str().unwrap());
    assert!(test_dll_path.is_dir());

    println!("Starting cargo build lib");
    let res = Command::new("cargo")
        .args(["build", "--lib", &format!("--target={target_triple}")])
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

    let relative_path_to_test_dll_file = &format!(
        "../../../../../bin/create_file_dll/target/{target_triple}/debug/create_file_dll.dll"
    );
    let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll_file);
    assert!(test_dll_path.is_file());
    println!("cargo:rustc-env=TEST_DLL_PATH={}", relative_path_to_test_dll_file);
}

fn build_bin_reflective_loader() {
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
    };

    let cargo_root = env!("CARGO_MANIFEST_DIR");
    // Hardcoded target for the reflective loader as per manual instructions
    let loader_target_triple = "x86_64-pc-windows-msvc";

    let reflective_loader_path_str = "../../../../../bin/reflective_loader";

    // Define triggers for rebuild
    let loader_files = ["src/lib.rs", "src/loader.rs", "Cargo.toml"];
    for f in loader_files {
        let binding = format!("{}/{}", reflective_loader_path_str, f);
        let tmp_path = Path::new(cargo_root).join(binding.as_str());
        // Only trigger rerun if the source file actually exists
        if let Ok(canon_path) = tmp_path.canonicalize() {
            println!("cargo:rerun-if-changed={}", canon_path.to_str().unwrap());
        }
    }

    // Get the absolute path of the reflective_loader workspace member
    let loader_root_path = Path::new(cargo_root)
        .join(reflective_loader_path_str)
        .canonicalize()
        .expect("Could not find reflective_loader directory");

    assert!(loader_root_path.is_dir());

    println!("Starting cargo xwin build for reflective_loader");

    // Command:
    // cargo xwin build --release \
    // -Z build-std=core,compiler_builtins \
    // -Z build-std-features=compiler-builtins-mem \
    // --target x86_64-pc-windows-msvc

    let res_build = Command::new("cargo")
        .args([
            "+nightly", // -Z flags require nightly
            "xwin",
            "build",
            "--release",
            "-Z",
            "build-std=core,compiler_builtins",
            "-Z",
            "build-std-features=compiler-builtins-mem",
            "--target",
            loader_target_triple,
        ])
        .current_dir(&loader_root_path)
        // Clean environment to prevent host compilation flags from leaking into target compilation
        .env_remove("TARGET")
        .env_remove("CARGO")
        .env_remove("RUSTC")
        .env_remove("RUSTUP_TOOLCHAIN")
        // We generally want to remove CARGO_TARGET_DIR so the nested cargo uses its own target dir
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_MANIFEST_DIR")
        .env(
            "RUSTFLAGS",
            "-C target-feature=+crt-static -C link-arg=/FIXED",
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn cargo xwin build")
        .stderr
        .unwrap();

    let reader = BufReader::new(res_build);
    reader
        .lines()
        .map_while(Result::ok)
        .for_each(|line| println!("cargo loader build: {}", line));

    // Verify the file exists at the expected location
    // ../../../../../bin/reflective_loader/target/x86_64-pc-windows-msvc/release/reflective_loader.dll
    let relative_path_to_loader_dll = format!(
        "{}/target/{}/release/reflective_loader.dll",
        reflective_loader_path_str, loader_target_triple
    );

    let loader_dll_path = Path::new(cargo_root).join(relative_path_to_loader_dll);
    assert!(
        loader_dll_path.exists(),
        "reflective_loader.dll not found at expected path: {:?}",
        loader_dll_path
    );
}

#[cfg(windows)]
const HOST_FAMILY: &str = "windows";

#[cfg(unix)]
const HOST_FAMILY: &str = "unix";

fn set_host_family() {
    println!("cargo::rustc-check-cfg=cfg(host_family, values(\"unix\", \"windows\"))");
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
