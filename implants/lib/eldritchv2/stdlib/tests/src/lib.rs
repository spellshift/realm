#[cfg(test)]
mod tests {
    use anyhow::Result;
    use eldritchv2::{Interpreter, Value};
    use glob::glob;
    use serde::Deserialize;
    use spin::RwLock;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Deserialize)]
    struct Metadata {
        paramdefs: Option<Vec<ParamDef>>,
    }

    #[derive(Deserialize)]
    struct ParamDef {
        name: String,
        #[serde(rename = "type")]
        type_: String,
    }

    fn register_fake_libs(interp: &mut Interpreter) {
        interp.register_lib(eldritchv2::agent::fake::AgentLibraryFake);
        interp.register_lib(eldritchv2::assets::fake::FakeAssetsLibrary);
        interp.register_lib(eldritchv2::file::fake::FileLibraryFake::default());
        interp.register_lib(eldritchv2::http::fake::HttpLibraryFake);
        interp.register_lib(eldritchv2::pivot::fake::PivotLibraryFake);
        interp.register_lib(eldritchv2::process::fake::ProcessLibraryFake);
        interp.register_lib(eldritchv2::report::fake::ReportLibraryFake);
        interp.register_lib(eldritchv2::sys::fake::SysLibraryFake);
    }

    fn register_default_libs(interp: &mut Interpreter) {
        // Register non-fake libs that are safe to use
        interp.register_lib(eldritchv2::crypto::std::StdCryptoLibrary);
        interp.register_lib(eldritchv2::random::std::StdRandomLibrary);
        interp.register_lib(eldritchv2::regex::std::StdRegexLibrary);
        interp.register_lib(eldritchv2::time::std::StdTimeLibrary);
        interp.register_lib(eldritchv2::cache::std::StdCacheLibrary::new());
    }

    #[test]
    fn test_cache_library() -> Result<()> {
        // Create a shared cache library instance
        let cache_lib = eldritchv2::cache::std::StdCacheLibrary::new();

        let mut interp1 = Interpreter::new();
        // Register the shared instance (assuming StdCacheLibrary is Clone and shares state internally via Arc)
        interp1.register_lib(cache_lib.clone());

        let code1 = r#"
cache.set("foo", "bar")
return cache.get("foo")
"#;

        let res1 = interp1.interpret(code1).map_err(|e| anyhow::anyhow!(e))?;
        if let Value::String(s) = res1 {
            assert_eq!(s, "bar");
        } else {
            panic!("Expected string 'bar', got {:?}", res1);
        }

        // Test shared state across interpreters
        let mut interp2 = Interpreter::new();
        // Register the SAME cache instance
        interp2.register_lib(cache_lib);

        let code2 = r#"
return cache.get("foo")
"#;

        let res2 = interp2.interpret(code2).map_err(|e| anyhow::anyhow!(e))?;
        if let Value::String(s) = res2 {
            assert_eq!(s, "bar");
        } else {
            panic!("Expected string 'bar' in interp2, got {:?}", res2);
        }

        // Test delete
        let code3 = r#"
cache.delete("foo")
return cache.get("foo")
"#;
        let res3 = interp2.interpret(code3).map_err(|e| anyhow::anyhow!(e))?;
        assert!(matches!(res3, Value::None));

        Ok(())
    }

    #[test]
    fn test_all_tomes() -> Result<()> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
        let root_dir = PathBuf::from(manifest_dir).join("../../../../../tavern/tomes");

        let pattern = root_dir.join("*/metadata.yml");
        let entries = glob(pattern.to_str().unwrap())?;

        let mut errors = Vec::new();

        for entry in entries {
            let path = entry?;
            println!(
                "Testing tome: {:?}",
                path.parent().unwrap().file_name().unwrap()
            );
            if let Err(e) = run_tome_test(&path) {
                println!(
                    "FAILED: {:?} - {}",
                    path.parent().unwrap().file_name().unwrap(),
                    e
                );
                errors.push((path, e));
            }
        }

        if !errors.is_empty() {
            panic!(
                "{} tomes failed: {:?}",
                errors.len(),
                errors
                    .iter()
                    .map(|(p, _)| p.parent().unwrap().file_name().unwrap())
                    .collect::<Vec<_>>()
            );
        }
        Ok(())
    }

    fn run_tome_test(metadata_path: &PathBuf) -> Result<()> {
        let dir = metadata_path.parent().unwrap();
        let eldritch_path = dir.join("main.eldritch");

        if !eldritch_path.exists() {
            println!("Skipping {:?} (no main.eldritch)", dir);
            return Ok(());
        }

        let metadata_content = fs::read_to_string(metadata_path)?;
        let metadata: Metadata = serde_yaml::from_str(&metadata_content)?;

        let code = fs::read_to_string(&eldritch_path)?.replace("\r\n", "\n");

        let mut interp = Interpreter::new();
        register_fake_libs(&mut interp);
        register_default_libs(&mut interp);

        // Prepare input_params
        #[allow(clippy::mutable_key_type)]
        let mut input_params = BTreeMap::new();
        if let Some(params) = metadata.paramdefs {
            for param in params {
                // Logic to handle specific params to avoid crashes
                let val = if param.name.contains("path") || param.type_ == "file" {
                    Value::String("/tmp".to_string())
                } else if param.name.contains("port") {
                    // covers "ports"
                    Value::String("80,443".to_string())
                } else if param.name == "time" || param.name == "interval" {
                    Value::String("10".to_string()) // Some tomes expect string and convert to int
                } else {
                    match param.type_.as_str() {
                        "string" => Value::String("test_val".to_string()),
                        "int" | "integer" => Value::Int(1),
                        // Some tomes treat boolean as string "true"/"false" and call .lower()
                        // To be safe, let's provide string "true".
                        // If the tome treats it as bool, it might fail if it expects bool ops.
                        // But most use cases seen so far are .lower().
                        "bool" | "boolean" => Value::String("true".to_string()),
                        "float" => Value::Float(1.0),
                        _ => Value::String("default".to_string()),
                    }
                };
                input_params.insert(Value::String(param.name), val);
            }
        }
        let input_params_val = Value::Dictionary(Arc::new(RwLock::new(input_params)));
        interp.define_variable("input_params", input_params_val);

        match interp.interpret(&code) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Eldritch error: {}", e)),
        }
    }
}
