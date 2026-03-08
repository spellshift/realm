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
    let parsed: serde_json::Value = match serde_json::from_str(&val) {
        Ok(v) => v,
        Err(e) => panic!("IMIX_UNIQUE is not valid JSON: {e}\nValue was: {val}"),
    };

    let arr = parsed
        .as_array()
        .unwrap_or_else(|| panic!("IMIX_UNIQUE must be a JSON array\nValue was: {val}"));

    for (i, elem) in arr.iter().enumerate() {
        let obj = elem
            .as_object()
            .unwrap_or_else(|| panic!("IMIX_UNIQUE[{i}] must be a JSON object\nValue was: {elem}"));

        if !obj.contains_key("type") {
            panic!("IMIX_UNIQUE[{i}] is missing required \"type\" field\nValue was: {elem}");
        }

        for key in obj.keys() {
            if key != "type" && key != "args" {
                panic!(
                    "IMIX_UNIQUE[{i}] contains unexpected key \"{key}\"; only \"type\" and \"args\" are allowed\nValue was: {elem}"
                );
            }
        }
    }

    println!("cargo:rustc-env=IMIX_UNIQUE={val}");
}
