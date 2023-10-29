pub mod file;
pub mod process;
pub mod sys;
pub mod pivot;
pub mod assets;
pub mod crypto;

use std::sync::mpsc::Sender;
use serde_json::Map;
use starlark::collections::SmallMap;
use starlark::{starlark_module, PrintHandler};
use starlark::environment::{GlobalsBuilder, Module, Globals, LibraryExtension};
use starlark::syntax::{AstModule, Dialect};
use starlark::eval::Evaluator;
use starlark::values::dict::Dict;
use starlark::values::{Value, AllocValue};

use file::FileLibrary;
use process::ProcessLibrary;
use sys::SysLibrary;
use assets::AssetsLibrary;
use pivot::PivotLibrary;
use crate::crypto::CryptoLibrary;

pub fn get_eldritch() -> anyhow::Result<Globals> {
    #[starlark_module]
    fn eldritch(builder: &mut GlobalsBuilder) {
        const file: FileLibrary = FileLibrary();
        const process: ProcessLibrary = ProcessLibrary();
        const sys: SysLibrary = SysLibrary();
        const pivot: PivotLibrary = PivotLibrary();
        const assets: AssetsLibrary = AssetsLibrary();
        const crypto: CryptoLibrary = CryptoLibrary();
    }

    let globals = GlobalsBuilder::extended_by(
        &[
            LibraryExtension::StructType,
            LibraryExtension::RecordType,
            LibraryExtension::EnumType,
            LibraryExtension::Map,
            LibraryExtension::Filter,
            LibraryExtension::Partial,
            LibraryExtension::ExperimentalRegex,
            LibraryExtension::Debug,
            LibraryExtension::Print,
            LibraryExtension::Breakpoint,
            LibraryExtension::Json,
            LibraryExtension::Abs,
            LibraryExtension::Typing,
        ]
    ).with(eldritch).build();
    return Ok(globals);
}

pub struct EldritchPrintHandler{
    pub sender: Sender<String>,
}

impl PrintHandler for EldritchPrintHandler {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        let res = match self.sender.send(text.to_string()) {
            Ok(local_res) => local_res,
            Err(local_err) => return Err(anyhow::anyhow!(local_err.to_string())),
        };
        Ok(res)
    }
}

pub struct StdPrintHandler {
}

impl PrintHandler for StdPrintHandler {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        println!("{}", text.to_owned());
        Ok(())
    }
}


pub fn eldritch_run(tome_filename: String, tome_contents: String, tome_parameters: Option<String>, print_handler: &(dyn PrintHandler)) -> anyhow::Result<String> {
    // Boilder plate
    let ast =  match AstModule::parse(
            &tome_filename,
            tome_contents.as_str().to_owned(),
            &Dialect::Extended
        ) {
            Ok(res) => res,
            Err(err) => return Err(anyhow::anyhow!("[eldritch] Unable to parse eldritch tome: {}: {} {}", err.to_string(), tome_filename.as_str(), tome_contents.as_str())),
    };

    let tome_params_str: String = match tome_parameters {
        Some(local_param_string) => match local_param_string.as_str() {
            "" => "{}".to_string(), // If we get "" as our params update it to "{}"
            _ => local_param_string // Otherwise return our string.
        },
        None => "{}".to_string(),
    };

    let globals = match get_eldritch() {
        Ok(local_globals) => local_globals,
        Err(local_error) => return Err(anyhow::anyhow!("[eldritch] Failed to get_eldritch globals: {}", local_error.to_string())),
    };

    let module: Module = Module::new();

    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut input_params: Dict = Dict::new(res);

    let parsed: serde_json::Value = match serde_json::from_str(&tome_params_str){
        Ok(local_value) => local_value,
        Err(local_err) => return Err(anyhow::anyhow!("[eldritch] Error decoding tome_params to JSON: {}: {}", local_err.to_string(), tome_params_str)),
    };

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
            new_value = module.heap().alloc(tmp_list);
        }else if value.is_u64() || value.is_i64() || value.is_f64() || value.is_number() {
            // Down cast the number to i32. On failure return max i32.
            let tmp_value: i32 = match value.as_i64() {
                Some(tmp_i64) => match tmp_i64.try_into() {
                    Ok(tmp_i32) => tmp_i32,
                    Err(_) => i32::MAX.into(),
                },
                None => i32::MAX.into(),
            };
            new_value = module.heap().alloc(tmp_value);
        }
        let hashed_key = match new_key.to_value().get_hashed() {
            Ok(local_hashed_key) => local_hashed_key,
            Err(local_error) => return Err(anyhow::anyhow!("[eldritch] Failed to create hashed key for key {}: {}", new_key.to_string(), local_error.to_string())),
        };
        input_params.insert_hashed(hashed_key, new_value);
    }
    module.set("input_params", input_params.alloc_value(module.heap()));

    let mut eval: Evaluator = Evaluator::new(&module);
    eval.set_print_handler(print_handler);

    let res: Value = match eval.eval_module(ast, &globals) {
        Ok(eval_val) => eval_val,
        Err(eval_error) => return Err(anyhow::anyhow!("[eldritch] Eldritch eval_module failed:\n{}", eval_error)),
    };

    Ok(res.to_str())
}

#[cfg(test)]
mod tests {
    use std::{thread, sync::mpsc::{channel}, time::Duration};

    use super::*;
    use starlark::assert::Assert;
    use tempfile::NamedTempFile;

    // just checks dir...
    #[test]
    fn test_library_bindings() {
        let globals = get_eldritch().unwrap();
        let mut a = Assert::new();
        a.globals(globals);
        a.all_true(
            r#"
dir(file) == ["append", "compress", "copy", "download", "exists", "hash", "is_dir", "is_file", "list", "mkdir", "read", "remove", "rename", "replace", "replace_all", "template", "timestomp", "write"]
dir(process) == ["info", "kill", "list", "name", "netstat"]
dir(sys) == ["dll_inject", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_user", "hostname", "is_linux", "is_macos", "is_windows", "shell"]
dir(pivot) == ["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_copy", "ssh_exec", "ssh_password_spray"]
dir(assets) == ["copy","list","read","read_binary"]
dir(crypto) == ["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "to_json"]
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
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string), &StdPrintHandler{});
        assert!(test_res?.contains("hello_world"));
        Ok(())
    }

    #[test]
    fn test_library_parameter_input_number() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
input_params["number"]
"#);
        let param_string = r#"{"number":1}"#.to_string();
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string), &StdPrintHandler{});
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
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string), &StdPrintHandler{});
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
        let test_res = eldritch_run("test.tome".to_string(), test_content, Some(param_string), &StdPrintHandler{});
        assert_eq!(test_res.unwrap(), r#"{"list_key": ["item1", "item2", "item3"]}"#);
        Ok(())
    }

    #[tokio::test]
    async fn test_library_async() -> anyhow::Result<()> {
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone().replace("\\", "\\\\");
        let test_content = format!(r#"
file.download("https://www.google.com/", "{path}")
"#);
        let test_res = thread::spawn(|| { eldritch_run("test.tome".to_string(), test_content, None, &StdPrintHandler{}) });
        let _test_val = test_res.join();

        assert!(tmp_file.as_file().metadata().unwrap().len() > 5);

        Ok(())
    }
    #[tokio::test]
    async fn test_library_custom_print_handler() -> anyhow::Result<()> {
        // just using a temp file for its path
        let test_content = format!(r#"
print("Hello")
print("World")
print("123")
"#);
        let (sender, receiver) = channel::<String>();

        let test_res = thread::spawn(|| { eldritch_run("test.tome".to_string(), test_content, None, &EldritchPrintHandler{ sender }) });
        let _test_val = test_res.join();
        let expected_output = vec!["Hello", "World", "123"];
        let mut index = 0;
        loop {
            let res = match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(local_res_string) => local_res_string,
                Err(local_err) => {
                    match local_err.to_string().as_str() {
                        "channel is empty and sending half is closed" => { break; },
                        _ => eprint!("Error: {}", local_err),
                    }
                    break;
                },
            };
            assert_eq!(res, expected_output[index].to_string());
            index = index + 1;
        }

        Ok(())
    }

}
