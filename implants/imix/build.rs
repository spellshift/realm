fn main() {
    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();

    if std::env::var("CARGO_FEATURE_TOKIO_CONSOLE").is_ok() {
        println!("cargo:rustc-cfg=tokio_unstable");
    }

    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();
    if profile == "debug" || imix_debug == "true" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }
}
