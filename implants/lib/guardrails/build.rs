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

    if let Err(msg) = validate_guardrails_json(&val) {
        panic!("{msg}");
    }
}

fn validate_guardrails_json(val: &str) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(val)
        .map_err(|e| format!("IMIX_GUARDRAILS is not valid JSON: {e}\nValue was: {val}"))?;

    let arr = parsed
        .as_array()
        .ok_or_else(|| format!("IMIX_GUARDRAILS must be a JSON array\nValue was: {val}"))?;

    for (i, elem) in arr.iter().enumerate() {
        let obj = elem.as_object().ok_or_else(|| {
            format!("IMIX_GUARDRAILS[{i}] must be a JSON object\nValue was: {elem}")
        })?;

        for key in obj.keys() {
            if key != "type" && key != "args" {
                return Err(format!(
                    "IMIX_GUARDRAILS[{i}] contains unexpected key \"{key}\"; only \"type\" and \"args\" are allowed\nValue was: {elem}"
                ));
            }
        }

        let gtype = obj.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            format!(
                "IMIX_GUARDRAILS[{i}] is missing required string \"type\" field\nValue was: {elem}"
            )
        })?;

        let args = obj.get("args").and_then(|v| v.as_object()).ok_or_else(|| {
            format!(
                "IMIX_GUARDRAILS[{i}] is missing required object \"args\" field\nValue was: {elem}"
            )
        })?;

        match gtype {
            "file" => {
                for key in args.keys() {
                    if key != "path" {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (file) contains unexpected arg \"{key}\"; only \"path\" is allowed\nValue was: {elem}"
                        ));
                    }
                }
                match args.get("path").and_then(|v| v.as_str()) {
                    Some(p) if !p.is_empty() => {}
                    _ => {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (file) requires non-empty string args.path\nValue was: {elem}"
                        ));
                    }
                }
            }
            "process" => {
                for key in args.keys() {
                    if key != "name" {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (process) contains unexpected arg \"{key}\"; only \"name\" is allowed\nValue was: {elem}"
                        ));
                    }
                }
                match args.get("name").and_then(|v| v.as_str()) {
                    Some(n) if !n.is_empty() => {}
                    _ => {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (process) requires non-empty string args.name\nValue was: {elem}"
                        ));
                    }
                }
            }
            "registry" => {
                for key in args.keys() {
                    if key != "path" && key != "subkey" && key != "value_name" {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (registry) contains unexpected arg \"{key}\"; only \"path\" (or \"subkey\") and \"value_name\" are allowed\nValue was: {elem}"
                        ));
                    }
                }
                let path_val = args.get("path").or_else(|| args.get("subkey"));
                match path_val.and_then(|v| v.as_str()) {
                    Some(p) if !p.is_empty() => {}
                    _ => {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (registry) requires non-empty string args.path\nValue was: {elem}"
                        ));
                    }
                }
                if let Some(vn) = args.get("value_name") {
                    if !vn.is_null() && !vn.is_string() {
                        return Err(format!(
                            "IMIX_GUARDRAILS[{i}] (registry) args.value_name must be a string or null\nValue was: {elem}"
                        ));
                    }
                }
            }
            other => {
                return Err(format!(
                    "IMIX_GUARDRAILS[{i}] has unknown guardrail type \"{other}\"; expected \"file\", \"process\", or \"registry\"\nValue was: {elem}"
                ));
            }
        }
    }

    Ok(())
}
