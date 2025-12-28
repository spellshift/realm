fn main() {
    // Run the config builder to parse IMIX_CONFIG and emit build directives
    let status = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            "../../bin/imix-config-builder/Cargo.toml",
        ])
        .status()
        .expect("Failed to run imix-config-builder");

    if !status.success() {
        panic!("imix-config-builder failed with status: {}", status);
    }

    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();
}
