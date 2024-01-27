use crate::pb::{File, ProcessList};
use crate::{
    assets::AssetsLibrary, crypto::CryptoLibrary, file::FileLibrary, pb::Tome, pivot::PivotLibrary,
    process::ProcessLibrary, sys::SysLibrary, time::TimeLibrary,
};
use anyhow::{Context, Error, Result};
use chrono::Utc;
use prost_types::Timestamp;
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
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

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

    ch_exec_started_at: Sender<Timestamp>,
    ch_exec_finished_at: Sender<Timestamp>,
    ch_output: Sender<String>,
    ch_error: Sender<Error>,
    ch_process_list: Sender<ProcessList>,
    ch_file: Sender<File>,
}

impl Runtime {
    /*
     * Prepare a new Runtime for execution of a single tome.
     */
    pub fn new() -> (Runtime, Output) {
        let (ch_exec_started_at, exec_started_at) = channel::<Timestamp>();
        let (ch_exec_finished_at, exec_finished_at) = channel::<Timestamp>();
        let (ch_error, errors) = channel::<Error>();
        let (ch_output, outputs) = channel::<String>();
        let (ch_process_list, process_lists) = channel::<ProcessList>();
        let (ch_file, files) = channel::<File>();

        return (
            Runtime {
                stdout_reporting: false,
                ch_exec_started_at,
                ch_exec_finished_at,
                ch_output,
                ch_error,
                ch_process_list,
                ch_file,
            },
            Output {
                exec_started_at,
                exec_finished_at,
                outputs,
                errors,
                process_lists,
                files,
            },
        );
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
    pub fn run(&self, tome: Tome) {
        match self.report_exec_started_at() {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_started_at: {_err}");
            }
        }

        match self.run_impl(tome) {
            Ok(_) => {}
            Err(err) => match self.report_error(err) {
                Ok(_) => {}
                Err(_send_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to send error: {_send_err}");
                }
            },
        }

        match self.report_exec_finished_at() {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send exec_finished_at: {_err}");
            }
        }
    }

    fn run_impl(&self, tome: Tome) -> Result<()> {
        let ast = Runtime::parse(&tome)?;
        let module = Runtime::alloc_module(&tome)?;
        let globals = Runtime::globals();

        let mut eval: Evaluator = Evaluator::new(&module);
        eval.extra = Some(self);
        eval.set_print_handler(self);

        match eval.eval_module(ast, &globals) {
            Ok(_) => Ok(()),
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("tome execution failed: {_err}");
                Err(_err)
            }
        }
    }

    /*
     * Globals available to eldritch code.
     * This provides all of our starlark standard libraries.
     */
    pub fn globals() -> Globals {
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
    pub fn with_stdout_reporting(&mut self) -> &mut Self {
        self.stdout_reporting = true;
        self
    }

    /*
     * Send exec_started_at timestamp.
     */
    fn report_exec_started_at(&self) -> Result<()> {
        let now = Utc::now();
        self.ch_exec_started_at.send(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        })?;
        Ok(())
    }

    /*
     * Send exec_finished_at timestamp.
     */
    fn report_exec_finished_at(&self) -> Result<()> {
        let now = Utc::now();
        self.ch_exec_finished_at.send(Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        })?;
        Ok(())
    }

    /*
     * Report output of the tome execution.
     */
    pub fn report_output(&self, output: String) -> Result<()> {
        self.ch_output.send(output)?;
        Ok(())
    }

    /*
     * Report error of the tome execution.
     */
    pub fn report_error(&self, err: anyhow::Error) -> Result<()> {
        self.ch_error.send(err)?;
        Ok(())
    }

    /*
     * Report a process list that was collected by the tome.
     */
    pub fn report_process_list(&self, processes: ProcessList) -> Result<()> {
        self.ch_process_list.send(processes)?;
        Ok(())
    }

    /*
     * Report a file that was collected by the tome.
     */
    pub fn report_file(&self, f: File) -> Result<()> {
        self.ch_file.send(f)?;
        Ok(())
    }
}

/*
 * Enables Runtime to be used as a starlark print handler.
 */
impl PrintHandler for Runtime {
    fn println(&self, text: &str) -> anyhow::Result<()> {
        self.report_output(text.to_string())?;
        if self.stdout_reporting {
            print!("{}", text);
        }
        Ok(())
    }
}

/*
 * Output enables callers to listen for various types of output from the runtime.
 * Each of the `collect` methods will return lists of all currently available data.
 */
pub struct Output {
    exec_started_at: Receiver<Timestamp>,
    exec_finished_at: Receiver<Timestamp>,
    outputs: Receiver<String>,
    errors: Receiver<Error>,
    process_lists: Receiver<ProcessList>,
    files: Receiver<File>,
}

impl Output {
    /*
     * Returns the timestamp of when execution started, if available.
     */
    pub fn get_exec_started_at(&self) -> Option<Timestamp> {
        drain_last(&self.exec_started_at)
    }

    /*
     * Returns the timestamp of when execution finished, if available.
     */
    pub fn get_exec_finished_at(&self) -> Option<Timestamp> {
        drain_last(&self.exec_finished_at)
    }

    /*
     * Collects all currently available reported text output.
     */
    pub fn collect(&self) -> Vec<String> {
        drain(&self.outputs)
    }

    /*
     * Collects all currently available reported errors, if any.
     */
    pub fn collect_errors(&self) -> Vec<Error> {
        drain(&self.errors)
    }

    /*
     * Returns all currently available reported process lists, if any.
     */
    pub fn collect_process_lists(&self) -> Vec<ProcessList> {
        drain(&self.process_lists)
    }

    /*
     * Returns all currently available reported files, if any.
     */
    pub fn collect_files(&self) -> Vec<File> {
        drain(&self.files)
    }
}

/*
 * Drain a receiver, returning only the last currently available result.
 */
fn drain_last<T>(receiver: &Receiver<T>) -> Option<T> {
    drain(receiver).pop()
}

/*
 * Drain a receiver, returning all currently available results as a Vec.
 */
fn drain<T>(reciever: &Receiver<T>) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    loop {
        let val = match reciever.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => v,
            Err(err) => {
                match err.to_string().as_str() {
                    "channel is empty and sending half is closed" => {
                        break;
                    }
                    "timed out waiting on channel" => {
                        break;
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        eprint!("failed to drain channel: {}", err)
                    }
                }
                break;
            }
        };
        // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
        result.push(val);
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::{pb::Tome, Runtime};
    use anyhow::Error;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    macro_rules! runtime_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let tc: TestCase = $value;
                let (runtime, output) = Runtime::new();
                runtime.run(tc.tome);

                let want_err_str = match tc.want_error {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                let err_str = match output.collect_errors().pop() {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                assert_eq!(want_err_str, err_str);
                assert_eq!(tc.want_output, output.collect().join(""));
            }
        )*
        }
    }

    struct TestCase {
        pub tome: Tome,
        pub want_output: String,
        pub want_error: Option<Error>,
    }

    runtime_tests! {
        simple_run: TestCase{
            tome: Tome{
                eldritch: String::from("print(1+1)"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from("2"),
            want_error: None,
        },
        multi_print: TestCase {
            tome: Tome{
                eldritch: String::from(r#"print("oceans "); print("rise, "); print("empires "); print("fall")"#),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"oceans rise, empires fall"#),
            want_error: None,
        },
        input_params: TestCase{
            tome: Tome {
                            eldritch: r#"print(input_params['cmd2'])"#.to_string(),
                            parameters: HashMap::from([
                                ("cmd".to_string(), "id".to_string()),
                                ("cmd2".to_string(), "echo hello_world".to_string()),
                                ("cmd3".to_string(), "ls -lah /tmp/".to_string()),
                            ]),
                            file_names: Vec::new(),
                        },
                        want_output: String::from("echo hello_world"),
                        want_error: None,
        },
        file_bindings: TestCase {
            tome: Tome {
                eldritch: String::from("print(dir(file))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["append", "compress", "copy", "download", "exists", "find", "is_dir", "is_file", "list", "mkdir", "moveto", "read", "remove", "replace", "replace_all", "template", "timestomp", "write"]"#),
            want_error: None,
        },
        process_bindings: TestCase {
            tome: Tome{
                eldritch: String::from("print(dir(process))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["info", "kill", "list", "name", "netstat"]"#),
            want_error: None,
        },
        sys_bindings: TestCase {
            tome: Tome{
                eldritch: String::from("print(dir(sys))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["dll_inject", "dll_reflect", "exec", "get_env", "get_ip", "get_os", "get_pid", "get_reg", "get_user", "hostname", "is_linux", "is_macos", "is_windows", "shell", "write_reg_hex", "write_reg_int", "write_reg_str"]"#),
            want_error: None,
        },
        pivot_bindings: TestCase {
            tome: Tome {
                eldritch: String::from("print(dir(pivot))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["arp_scan", "bind_proxy", "ncat", "port_forward", "port_scan", "smb_exec", "ssh_copy", "ssh_exec", "ssh_password_spray"]"#),
            want_error: None,
        },
        assets_bindings: TestCase {
            tome: Tome {
                eldritch: String::from("print(dir(assets))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["copy", "list", "read", "read_binary"]"#),
            want_error: None,
        },
        crypto_bindings: TestCase {
            tome: Tome {
                eldritch: String::from("print(dir(crypto))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["aes_decrypt_file", "aes_encrypt_file", "decode_b64", "encode_b64", "from_json", "hash_file", "to_json"]"#),
            want_error: None,
        },
        time_bindings: TestCase {
            tome: Tome {
                eldritch: String::from("print(dir(time))"),
                parameters: HashMap::new(),
                file_names: Vec::new(),
            },
            want_output: String::from(r#"["format_to_epoch", "format_to_readable", "now", "sleep"]"#),
            want_error: None,
        },
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 128)]
    async fn test_library_async() -> anyhow::Result<()> {
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap())
            .clone()
            .replace("\\", "\\\\");
        let eldritch =
            format!(r#"file.download("https://www.google.com/", "{path}"); print("ok")"#);
        let (runtime, output) = Runtime::new();
        let t = tokio::task::spawn_blocking(move || {
            runtime.run(Tome {
                eldritch,
                parameters: HashMap::new(),
                file_names: Vec::new(),
            });
        });
        assert!(t.await.is_ok());

        let out = output.collect();
        let err = output.collect_errors().pop();
        assert!(
            err.is_none(),
            "failed with err {}",
            err.unwrap().to_string()
        );
        assert!(tmp_file.as_file().metadata().unwrap().len() > 5);
        assert_eq!("ok", out.join(""));
        Ok(())
    }
}
