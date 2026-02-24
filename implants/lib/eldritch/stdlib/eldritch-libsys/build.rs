fn main() {
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=crypt");
}
