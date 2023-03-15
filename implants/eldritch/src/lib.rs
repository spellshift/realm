pub mod file;
pub mod process;
pub mod sys;
pub mod pivot;

use serde_json::Map;
use starlark::collections::SmallMap;
use starlark::starlark_module;
use starlark::environment::{GlobalsBuilder, Module, Globals};
use starlark::syntax::{AstModule, Dialect};
use starlark::eval::Evaluator;
use starlark::values::dict::Dict;
use starlark::values::{Value, AllocValue};

use file::FileLibrary;
use process::ProcessLibrary;
use sys::SysLibrary;

pub fn get_eldritch() -> anyhow::Result<Globals> {
    #[starlark_module]
    fn eldritch(builder: &mut GlobalsBuilder) {
        const file: FileLibrary = FileLibrary();
        const process: ProcessLibrary = ProcessLibrary();
        const sys: SysLibrary = SysLibrary();
    }

    let globals = GlobalsBuilder::extended().with(eldritch).build();
    return Ok(globals);
}

pub fn eldritch_run(tome_filename: String, tome_contents: String, tome_parameters: Option<String>) -> anyhow::Result<String> {
    // Boilder plate
    let ast: AstModule;
    match AstModule::parse(
            &tome_filename,
            tome_contents.as_str().to_owned(),
            &Dialect::Standard
        ) {
            Ok(res) => ast = res,
            Err(err) => return Err(err),
    }

    let tome_params_str: String = match tome_parameters {
        Some(param_string) => param_string,
        None => "{}".to_string(),
    };

    let globals = get_eldritch()?;

    let module: Module = Module::new();

    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut input_params: Dict = Dict::new(res);
    
    let parsed: serde_json::Value = serde_json::from_str(&tome_params_str)?;
    let param_map: serde_json::Map<String, serde_json::Value> = match parsed.as_object() {
        Some(tmp_param_map) => tmp_param_map.clone(),
        None => Map::new(),
    };
    for (key, value) in param_map.iter() {
        let mut new_value: Value = Value::new_none();
        let new_key = module.heap().alloc_str(&key);

        if value.is_string() {
            let tmp_val = match value.as_str() {
                Some(tmp_str_val) => tmp_str_val,
                None => continue, // Failed to get value as str skip it.
            };
            new_value = module.heap().alloc_str(tmp_val).to_value();
        }else if value.is_array() {
            let mut tmp_list: Vec<Value> = Vec::new();
            let val_array = match value.as_array() {
                Some(val_array_vec) => val_array_vec,
                None => continue, // Failed to get value as str skip it.
            };
            for sub_value in val_array {
                if sub_value.is_string() {
                    let tmp_sub_value = match sub_value.as_str() {
                        Some(sub_value_str) => sub_value_str,
                        None => continue, // Failed to get value as str skip it.
                    };
                    tmp_list.push(module.heap().alloc_str(tmp_sub_value).to_value());
                }
            }
            new_value = module.heap().alloc_list(tmp_list.as_slice());
        }else if value.is_u64() || value.is_i64() || value.is_f64() || value.is_number() {
            // Down cast the number to i32. On failure return max i32.
            let tmp_value: i32 = match value.as_i64() {
                Some(tmp_i64) => match tmp_i64.try_into() {
                    Ok(tmp_i32) => tmp_i32,
                    Err(_) => i32::MAX.into(),
                },
                None => i32::MAX.into(),
            };
            new_value = Value::new_int(tmp_value);
        }
        let hashed_key = new_key.to_value().get_hashed()?;
        input_params.insert_hashed(hashed_key, new_value);
    }

    module.set("input_params", input_params.alloc_value(module.heap()));

    let mut eval: Evaluator = Evaluator::new(&module);
    let res: Value = match eval.eval_module(ast, &globals) {
        Ok(eval_val) => eval_val,
        Err(eval_error) => return Err(anyhow::anyhow!("Eldritch eval_module failed:\n{}", eval_error)),
    };

    Ok(res.to_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::environment::{GlobalsBuilder};
    use starlark::{starlark_module};
    use starlark::assert::Assert;

    use super::file::FileLibrary;
    use super::process::ProcessLibrary;
    use super::sys::SysLibrary;
    use super::pivot::PivotLibrary;

    // just checks dir...
    #[test]
    fn test_library_bindings() {
        #[starlark_module]
        fn globals(builder: &mut GlobalsBuilder) {
            const file: FileLibrary = FileLibrary();
            const process: ProcessLibrary = ProcessLibrary();
            const sys: SysLibrary = SysLibrary();
            const pivot: PivotLibrary = PivotLibrary();
        }

        let mut a = Assert::new();
        a.globals_add(globals);
        a.all_true(
            r#"
dir(file) == ["append", "compress", "copy", "download", "exists", "hash", "is_dir", "is_file", "mkdir", "read", "remove", "rename", "replace", "replace_all", "template", "timestomp", "write"]
dir(process) == ["kill", "list", "name"]
dir(sys) == ["dll_inject", "exec", "is_linux", "is_macos", "is_windows", "shell"]
dir(pivot) == ["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_exec", "ssh_password_spray"]
"#,
        );
    }
    
    #[test]
    fn test_library_parameter_input_string() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
sys.shell(input_params['cmd2'])
"#);
        let param_string = r#"{"cmd":"id","cmd2":"echo hello_world","cmd3":"ls -lah /tmp/"}"#.to_string();
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string));
        assert_eq!(test_res.unwrap().trim(), "hello_world".to_string());
        Ok(())
    }
    #[test]
    fn test_library_parameter_input_number() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
input_params["number"]
"#);
        let param_string = r#"{"number":1}"#.to_string();
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string));
        assert_eq!(test_res.unwrap(), "1".to_string());
        Ok(())
    }

    #[test]
    fn test_library_parameter_input_max_u64() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
x = input_params["number"] - 1
x
"#);
        let param_string = format!("{{\"number\":{}}}", u64::MAX);
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string));
        assert_eq!(test_res.unwrap(), "2147483646".to_string()); // i32::MAX-1
        Ok(())
    }

    #[test]
    fn test_library_parameter_input_str_array() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
input_params
"#);
        let param_string = r#"{"list_key":["item1","item2","item3"]}"#.to_string();
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string));
        assert_eq!(test_res.unwrap(), r#"{"list_key": ["item1", "item2", "item3"]}"#);
        Ok(())
    }

}