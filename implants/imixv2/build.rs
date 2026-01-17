fn main() {
    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();

    if std::env::var("CARGO_FEATURE_TOKIO_CONSOLE").is_ok() {
        println!("cargo:rustc-cfg=tokio_unstable");
    }
}
