fn main() {
    println!("cargo:rerun-if-env-changed=IMIX_GUARDRAILS");
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();

    if profile == "debug" || imix_debug == "tomes" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug_tome\"");
    }

    if profile == "debug" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }

    let val = match std::env::var("IMIX_GUARDRAILS") {
        Ok(v) => v,
        Err(_) => return,
    };

    let parsed: serde_json::Value = match serde_json::from_str(&val) {
        Ok(v) => v,
        Err(e) => panic!("IMIX_GUARDRAILS is not valid JSON: {e}\nValue was: {val}"),
    };

    let arr = parsed
        .as_array()
        .unwrap_or_else(|| panic!("IMIX_GUARDRAILS must be a JSON array\nValue was: {val}"));

    for (i, elem) in arr.iter().enumerate() {
        let obj = elem.as_object().unwrap_or_else(|| {
            panic!("IMIX_GUARDRAILS[{i}] must be a JSON object\nValue was: {elem}")
        });

        if !obj.contains_key("type") {
            panic!("IMIX_GUARDRAILS[{i}] is missing required \"type\" field\nValue was: {elem}");
        }

        for key in obj.keys() {
            if key != "type" && key != "args" {
                panic!(
                    "IMIX_GUARDRAILS[{i}] contains unexpected key \"{key}\"; only \"type\" and \"args\" are allowed\nValue was: {elem}"
                );
            }
        }
    }

    println!("cargo:rustc-env=IMIX_GUARDRAILS={val}");
}
