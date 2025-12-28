fn main() {
    // Run the config builder to parse IMIX_CONFIG and emit build directives
    if let Err(e) = imix_config_builder::run() {
        panic!("imix-config-builder failed: {}", e);
    }

    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();
}
