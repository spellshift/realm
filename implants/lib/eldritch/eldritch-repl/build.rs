fn main() {
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    if std::env::var("IMIX_DEBUG").unwrap_or_default() == "true" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }
}
