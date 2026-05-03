fn main() {
    // No documentation generation
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();

    if profile == "debug" || imix_debug == "tomes" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug_tome\"");
    }

    if profile == "debug" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }
}
