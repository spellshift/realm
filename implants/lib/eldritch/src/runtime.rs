use crate::channel::Sender;
use crate::{
    assets::AssetsLibrary, crypto::CryptoLibrary, file::FileLibrary, pb::Tome, pivot::PivotLibrary,
    process::ProcessLibrary, sys::SysLibrary, time::TimeLibrary,
};
use anyhow::{Context, Result};
use starlark::{
    collections::SmallMap,
    environment::{Globals, GlobalsBuilder, LibraryExtension, Module},
    eval::Evaluator,
    starlark_module,
    syntax::{AstModule, Dialect},
    values::{dict::Dict, AnyLifetime},
    values::{AllocValue, ProvidesStaticType},
    PrintHandler,
};

/*
 * Eldritch Runtime
 *
 * This runtime is responsible for executing Tomes and reporting their output.
 * It acts as an interface between callers and starlark, exposing our standard libraries to the starlark interpreter.
 * It is also used to provide dependency injection for eldritch library functions (using `Runtime::from_extra(starlark_interpreter.extra)`).
 */
#[derive(ProvidesStaticType)]
pub struct Runtime {
    stdout_reporting: bool,
    sender: Sender,
}

impl Runtime {
    /*
     * Prepare a new Runtime for execution of a single tome.
     */
    pub fn new(sender: Sender) -> Runtime {
        Runtime {
            stdout_reporting: false,
            sender,
        }
    }

    /*
     * Extract an existing runtime from the starlark evaluator extra field.
     */
    pub fn from_extra<'a>(extra: Option<&'a dyn AnyLifetime<'a>>) -> Result<&'a Runtime> {
        extra
            .context("no extra field present in evaluator")?
            .downcast_ref::<Runtime>()
            .context("no runtime present in evaluator")
    }

    /*
     * Run an Eldritch tome, returning an error if it fails.
     * Output from the tome is exposed via channels, see `reported_output`, `reported_process_list`, and `reported_files`.
     */
    pub fn run(&self, tome: Tome) -> Result<()> {
        let ast = Runtime::parse(&tome)?;
        let module = Runtime::alloc_module(&tome)?;
        let globals = Runtime::globals();

        let mut eval: Evaluator = Evaluator::new(&module);
        eval.extra = Some(self);
        eval.set_print_handler(self);

        eval.eval_module(ast, &globals)
            .context("tome execution failed")?;

        Ok(())
    }

    /*
     * Globals available to eldritch code.
     * This provides all of our starlark standard libraries.
     */
    fn globals() -> Globals {
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

        GlobalsBuilder::extended_by(&[
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
        .build()
    }

    /*
     * Parse an Eldritch tome into a starlark Abstract Syntax Tree (AST) Module.
     */
    fn parse(tome: &Tome) -> Result<AstModule> {
        match AstModule::parse("main", tome.eldritch.to_string(), &Dialect::Extended) {
            Ok(res) => Ok(res),
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "[eldritch] Unable to parse eldritch tome: {}: {}",
                    err.to_string(),
                    tome.eldritch.to_string(),
                ))
            }
        }
    }

    /*
     * Allocate tome parameters on a new starlark module and return it, ready for execution.
     */
    fn alloc_module(tome: &Tome) -> Result<Module> {
        let module: Module = Module::new();
        let mut input_params: Dict = Dict::new(SmallMap::new());

        for (key, value) in &tome.parameters {
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
        module.set("input_params", input_params.alloc_value(module.heap()));

        Ok(module)
    }

    /*
     * Print execution results to stdout as they become available.
     */
    pub fn with_stdout_reporting(mut self) {
        self.stdout_reporting = true;
    }
}

/*
 * Enables Runtime to be used as a starlark print handler.
 */
impl PrintHandler for Runtime {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        self.sender.report_output(text.to_string())?;
        if self.stdout_reporting {
            print!("{}", text);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use starlark::assert::Assert;
    use std::{collections::HashMap, thread};
    use tempfile::NamedTempFile;

    use crate::{channel::channel, pb::Tome, Runtime};

    #[test]
    fn test_run() -> Result<()> {
        let (sender, recv) = channel();
        let runtime = Runtime::new(sender);
        let result = runtime
            .run(Tome {
                eldritch: "print(1+1)".to_string(),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            })
            .is_ok();
        assert!(result);
        let output = recv.collect_output()?;
        assert_eq!("2", output.join(""));
        Ok(())
    }

    #[test]
    fn test_library_bindings() {
        let globals = Runtime::globals();
        let mut a = Assert::new();
        a.globals(globals);
        a.all_true(
            r#"
dir(file) == ["append", "compress", "copy", "download", "exists", "find", "is_dir", "is_file", "list", "mkdir", "moveto", "read", "remove", "replace", "replace_all", "template", "timestomp", "write"]
dir(process) == ["info", "kill", "list", "name", "netstat"]
dir(sys) == ["dll_inject", "dll_reflect", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_reg", "get_user", "hostname", "is_linux", "is_macos", "is_windows", "shell", "write_reg_hex", "write_reg_int", "write_reg_str"]
dir(pivot) == ["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_copy", "ssh_exec", "ssh_password_spray"]
dir(assets) == ["copy","list","read","read_binary"]
dir(crypto) == ["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "to_json"]
dir(time) == ["format_to_epoch", "format_to_readable", "now", "sleep"]
"#,
        );
    }

    #[test]
    fn test_collect_output() -> Result<()> {
        let (sender, recv) = channel();
        let runtime = Runtime::new(sender);
        let result = runtime
            .run(Tome {
                eldritch: r#"print("hello_world"); print("goodbye")"#.to_string(),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            })
            .is_ok();
        assert!(result);
        let output = recv.collect_output()?;
        assert_eq!("hello_world\ngoodbye", output.join("\n"));

        Ok(())
    }

    #[test]
    fn test_input_params() -> Result<()> {
        let (sender, recv) = channel();
        let runtime = Runtime::new(sender);
        let result = runtime
            .run(Tome {
                eldritch: r#"print(sys.shell(input_params['cmd2'])["stdout"])"#.to_string(),
                parameters: HashMap::from([
                    ("cmd".to_string(), "id".to_string()),
                    ("cmd2".to_string(), "echo hello_world".to_string()),
                    ("cmd3".to_string(), "ls -lah /tmp/".to_string()),
                ]),
                file_names: Vec::new(),
            })
            .is_ok();
        assert!(result);

        let output = recv.collect_output()?;
        assert_eq!("hello_world\n", output.join(""));
        Ok(())
    }

    #[tokio::test]
    async fn test_library_async() -> anyhow::Result<()> {
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap())
            .clone()
            .replace("\\", "\\\\");
        let eldritch = format!(
            r#"
file.download("https://www.google.com/", "{path}")
print("ok")
"#
        );
        let (sender, recv) = channel();
        let t = thread::spawn(|| {
            let runtime = Runtime::new(sender);

            let result = runtime
                .run(Tome {
                    eldritch,
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                })
                .is_ok();
            assert!(result);
        });
        assert!(t.join().is_ok());
        assert!(tmp_file.as_file().metadata().unwrap().len() > 5);
        let output = recv.collect_output()?;
        assert_eq!("ok", output.join(""));
        Ok(())
    }
}
