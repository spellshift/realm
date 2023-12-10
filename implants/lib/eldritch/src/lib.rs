pub mod assets;
pub mod crypto;
pub mod file;
pub mod pivot;
pub mod process;
pub mod sys;
pub mod time;

use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::environment::{Globals, GlobalsBuilder, LibraryExtension, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::dict::Dict;
use starlark::values::{AllocValue, Value};
use starlark::{starlark_module, PrintHandler};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::crypto::CryptoLibrary;
use assets::AssetsLibrary;
use file::FileLibrary;
use pivot::PivotLibrary;
use process::ProcessLibrary;
use sys::SysLibrary;
use time::TimeLibrary;

macro_rules! insert_dict_kv {
    ($dict:expr, $heap:expr, $key:expr, $val:expr, String) => {
        let val_val = $heap.alloc_str(&$val);
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            val_val.to_value(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, i32) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u32) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, u64) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, None) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            Value::new_none(),
        );
    };
    ($dict:expr, $heap:expr, $key:expr, $val:expr, Vec<_>) => {
        $dict.insert_hashed(
            const_frozen_string!($key).to_value().get_hashed()?,
            $heap.alloc($val),
        );
    };
}
pub(crate) use insert_dict_kv;

pub fn get_eldritch() -> anyhow::Result<Globals> {
    #[starlark_module]
    fn eldritch(builder: &mut GlobalsBuilder) {
        const file: FileLibrary = FileLibrary();
        const process: ProcessLibrary = ProcessLibrary();
        const sys: SysLibrary = SysLibrary();
        const pivot: PivotLibrary = PivotLibrary();
        const assets: AssetsLibrary = AssetsLibrary();
        const crypto: CryptoLibrary = CryptoLibrary();
        const time: TimeLibrary = TimeLibrary();
    }

    let globals = GlobalsBuilder::extended_by(&[
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
    ])
    .with(eldritch)
    .build();
    return Ok(globals);
}

pub struct EldritchPrintHandler {
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

pub struct StdPrintHandler {}

impl PrintHandler for StdPrintHandler {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        println!("{}", text.to_owned());
        Ok(())
    }
}

pub fn eldritch_run(
    tome_filename: String,
    tome_contents: String,
    tome_parameters: Option<HashMap<String, String>>,
    print_handler: &(dyn PrintHandler),
) -> anyhow::Result<String> {
    // Boilder plate
    let ast = match AstModule::parse(
        &tome_filename,
        tome_contents.as_str().to_owned(),
        &Dialect::Extended,
    ) {
        Ok(res) => res,
        Err(err) => {
            return Err(anyhow::anyhow!(
                "[eldritch] Unable to parse eldritch tome: {}: {} {}",
                err.to_string(),
                tome_filename.as_str(),
                tome_contents.as_str()
            ))
        }
    };

    // let tome_params_str: String = match tome_parameters {
    //     Some(local_param_string) => match local_param_string.as_str() {
    //         "" => "{}".to_string(),  // If we get "" as our params update it to "{}"
    //         _ => local_param_string, // Otherwise return our string.
    //     },
    //     None => "{}".to_string(),
    // };

    let globals = match get_eldritch() {
        Ok(local_globals) => local_globals,
        Err(local_error) => {
            return Err(anyhow::anyhow!(
                "[eldritch] Failed to get_eldritch globals: {}",
                local_error.to_string()
            ))
        }
    };

    let module: Module = Module::new();

    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut input_params: Dict = Dict::new(res);

    match tome_parameters {
        Some(params) => {
            for (key, value) in &params {
                let new_key = module.heap().alloc_str(&key);
                let new_value = module.heap().alloc_str(value.as_str()).to_value();
                let hashed_key = match new_key.to_value().get_hashed() {
                    Ok(local_hashed_key) => local_hashed_key,
                    Err(local_error) => {
                        return Err(anyhow::anyhow!(
                            "[eldritch] Failed to create hashed key for key {}: {}",
                            new_key.to_string(),
                            local_error.to_string()
                        ))
                    }
                };
                input_params.insert_hashed(hashed_key, new_value);
            }
        }
        None => {}
    }
    module.set("input_params", input_params.alloc_value(module.heap()));

    let mut eval: Evaluator = Evaluator::new(&module);
    eval.set_print_handler(print_handler);

    let res: Value = match eval.eval_module(ast, &globals) {
        Ok(eval_val) => eval_val,
        Err(eval_error) => {
            return Err(anyhow::anyhow!(
                "[eldritch] Eldritch eval_module failed:\n{}",
                eval_error
            ))
        }
    };

    Ok(res.to_str())
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::channel, thread, time::Duration};

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
dir(file) == ["append", "compress", "copy", "download", "exists", "hash", "is_dir", "is_file", "list", "mkdir", "moveto", "read", "remove", "replace", "replace_all", "template", "timestomp", "write"]
dir(process) == ["info", "kill", "list", "name", "netstat"]
dir(sys) == ["dll_inject", "dll_reflect", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_reg", "get_user", "hostname", "is_linux", "is_macos", "is_windows", "shell"]
dir(pivot) == ["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_copy", "ssh_exec", "ssh_password_spray"]
dir(assets) == ["copy","list","read","read_binary"]
dir(crypto) == ["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "to_json"]
dir(time) == ["sleep"]
"#,
        );
    }

    #[test]
    fn test_library_parameter_input_string() -> anyhow::Result<()> {
        // Create test script
        let test_content = format!(
            r#"
sys.shell(input_params['cmd2'])
"#
        );
        let params = HashMap::from([
            ("cmd".to_string(), "id".to_string()),
            ("cmd2".to_string(), "echo hello_world".to_string()),
            ("cmd3".to_string(), "ls -lah /tmp/".to_string()),
        ]);
        let test_res = eldritch_run(
            "test.tome".to_string(),
            test_content,
            Some(params),
            &StdPrintHandler {},
        );
        assert!(test_res?.contains("hello_world"));
        Ok(())
    }

    #[test]
    fn test_library_parameter_input_number() -> anyhow::Result<()> {
        // Create test script
        let test_content = format!(
            r#"
input_params["number"]
"#
        );
        let params = HashMap::from([("number".to_string(), "1".to_string())]);
        let test_res = eldritch_run(
            "test.tome".to_string(),
            test_content,
            Some(params),
            &StdPrintHandler {},
        );
        assert_eq!(test_res.unwrap(), "1".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_library_async() -> anyhow::Result<()> {
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap())
            .clone()
            .replace("\\", "\\\\");
        let test_content = format!(
            r#"
file.download("https://www.google.com/", "{path}")
"#
        );
        let test_res = thread::spawn(|| {
            eldritch_run(
                "test.tome".to_string(),
                test_content,
                None,
                &StdPrintHandler {},
            )
        });
        let _test_val = test_res.join();

        assert!(tmp_file.as_file().metadata().unwrap().len() > 5);

        Ok(())
    }
    #[tokio::test]
    async fn test_library_custom_print_handler() -> anyhow::Result<()> {
        // just using a temp file for its path
        let test_content = format!(
            r#"
print("Hello")
print("World")
print("123")
"#
        );
        let (sender, receiver) = channel::<String>();

        let test_res = thread::spawn(|| {
            eldritch_run(
                "test.tome".to_string(),
                test_content,
                None,
                &EldritchPrintHandler { sender },
            )
        });
        let _test_val = test_res.join();
        let expected_output = vec!["Hello", "World", "123"];
        let mut index = 0;
        loop {
            let res = match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(local_res_string) => local_res_string,
                Err(local_err) => {
                    match local_err.to_string().as_str() {
                        "channel is empty and sending half is closed" => {
                            break;
                        }
                        _ => eprint!("Error: {}", local_err),
                    }
                    break;
                }
            };
            assert_eq!(res, expected_output[index].to_string());
            index = index + 1;
        }

        Ok(())
    }
}
