use std::env;

fn main() {
    let _family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_else(|_| "unknown".to_string());
    // Since build.rs runs on the host, we need to check host family if we want to detect cross-compilation environment
    // But `CARGO_CFG_TARGET_FAMILY` is for the target.
    // The code in `dll_reflect_impl.rs` uses `host_family`.
    // It likely wants to know if the COMPILER (host) is windows or unix to decide path separators for `include_bytes!`.

    // Check HOST OS
    let host_os = env::consts::OS;
    let host_family = env::consts::FAMILY;

    println!("cargo:rustc-cfg=host_family=\"{}\"", host_family);
    println!("cargo:rustc-cfg=host_os=\"{}\"", host_os);

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "linux" {
        println!("cargo:rustc-link-lib=dylib=crypt");
    }
}
