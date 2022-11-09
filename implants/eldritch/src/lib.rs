pub mod file;
pub mod process;
pub mod sys;
pub mod pivot;

use anyhow::Error;

use starlark::{starlark_module};
use starlark::environment::{GlobalsBuilder, Module, Globals};
use starlark::syntax::{AstModule, Dialect};
use starlark::eval::Evaluator;
use starlark::values::Value;

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

pub fn eldritch_run(tome_filename: String, tome_contents: String) -> Result<String, Error> {
    let ast: AstModule = AstModule::parse(
        &tome_filename,
        tome_contents.as_str().to_owned(),
        &Dialect::Standard
    ).unwrap();

    let globals = get_eldritch()?;
    let module: Module = Module::new();

    let mut eval: Evaluator = Evaluator::new(&module);

    let res: Value = eval.eval_module(ast, &globals).unwrap();

    Ok(res.unpack_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use starlark::environment::{GlobalsBuilder};
    use starlark::{starlark_module};
    use starlark::assert::Assert;

    use super::file::FileLibrary;
    use super::process::ProcessLibrary;
    use super::sys::SysLibrary;

    // just checks dir...
    #[test]
    fn test_library_bindings() {
        #[starlark_module]
        fn globals(builder: &mut GlobalsBuilder) {
            const file: FileLibrary = FileLibrary();
            const process: ProcessLibrary = ProcessLibrary();
            const sys: SysLibrary = SysLibrary();
        }

        let mut a = Assert::new();
        a.globals_add(globals);
        a.all_true(
            r#"
dir(file) == ["append", "copy", "download", "exists", "hash", "is_dir", "is_file", "mkdir", "read", "remove", "rename", "replace", "replace_all", "timestomp", "write"]
dir(process) == ["kill", "list", "name"]
dir(sys) == ["exec", "is_linux", "is_macos", "is_windows", "shell"]
"#,
        );
    }
}