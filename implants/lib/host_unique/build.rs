fn main() {
    // Re-run this build script whenever IMIX_UNIQUE changes so the baked-in
    // value is always in sync with the environment.
    println!("cargo:rerun-if-env-changed=IMIX_UNIQUE");

    let val = match std::env::var("IMIX_UNIQUE") {
        Ok(v) => v,
        // Not set — nothing to bake in; the binary will fall back to defaults().
        Err(_) => return,
    };

    // Validate the value is well-formed JSON before baking it into the binary.
    // This catches configuration mistakes at compile time rather than at runtime.
    if let Err(e) = serde_json::from_str::<serde_json::Value>(&val) {
        panic!("IMIX_UNIQUE is not valid JSON: {e}\nValue was: {val}");
    }

    println!("cargo:rustc-env=IMIX_UNIQUE={val}");
}
