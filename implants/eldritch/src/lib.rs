pub mod file;
pub mod process;
pub mod sys;
pub mod pivot;

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
        None => "".to_string(),
    };

    let globals = get_eldritch()?;

    let module: Module = Module::new();

    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut input_vars: Dict = Dict::new(res);

    let parsed: serde_json::Value = serde_json::from_str(&tome_params_str)?;
    let obj: serde_json::Map<String, serde_json::Value> = parsed.as_object().unwrap().clone();
    for (key, value) in obj.iter() {
        let new_key = module.heap().alloc_str(&key);
        let new_value = module.heap().alloc_str(value.as_str().unwrap());
        input_vars.insert_hashed(new_key.to_value().get_hashed().unwrap(), new_value.to_value());
    }

    module.set("input_vars", input_vars.alloc_value(module.heap()));

    let mut eval: Evaluator = Evaluator::new(&module);
    let res: Value = match eval.eval_module(ast, &globals) {
        Ok(eval_val) => eval_val,
        Err(eval_error) => return Err(anyhow::anyhow!("Eldritch eval_module failed:\n{}", eval_error)),
    };

    let res_str = match res.unpack_str() {
        Some(res) => res.to_string(),
        None => return Err(anyhow::anyhow!("Failed to unpack result as str")),
    };

    Ok(res_str)
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
    fn test_library_parameter_input() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
sys.shell(input_vars['cmd2'])
"#);
        let param_string = r#"{"cmd":"id","cmd2":"echo hello_world","cmd3":"ls -lah /tmp/"}"#.to_string();
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string));
        assert_eq!(test_res.unwrap(), "hello_world\n".to_string());
        Ok(())
    }
}