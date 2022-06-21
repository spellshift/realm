pub mod file;
pub mod process;
pub mod sys;
pub mod pivot;

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
dir(file) == ["append", "copy", "download", "exists", "hash", "is_dir", "mkdir", "read", "remove", "rename", "replace", "replace_all", "timestomp", "write"]
dir(process) == ["kill", "list", "name"]
dir(sys) == ["exec", "is_linux", "is_macos", "is_windows", "shell"]
"#,
        );
    }
}