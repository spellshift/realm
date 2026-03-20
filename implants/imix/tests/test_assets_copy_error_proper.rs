use eldritch::Interpreter;
use eldritch::Value;
use eldritch::assets::std::StdAssetsLibrary;
use eldritch::conversion::ToValue;
use std::sync::Arc;

#[test]
fn test_assets_copy_error_proper() {
    println!("Hello from proper test");
    let mut interp = Interpreter::new().with_default_libs();

    // Add StdAssetsLibrary to interpreter
    let assets_lib = StdAssetsLibrary::new();
    // It's empty, so any copy will fail because read fails
    interp.define_variable("assets", Value::Foreign(Arc::new(assets_lib)));

    let code = r#"
assets.copy("nonexistent", "dest")
"#;
    let result = interp.interpret(code);
    println!("Interpreter Result: {:?}", result);
}
