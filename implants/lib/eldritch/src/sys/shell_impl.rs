use super::super::insert_dict_kv;
use anyhow::{Context, Result};
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::dict::Dict;
use starlark::values::Heap;
use std::process::Command;
use std::str;

use super::CommandOutput;

pub fn shell(starlark_heap: &Heap, cmd: String) -> Result<Dict> {
    let cmd_res = handle_shell(cmd)?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    insert_dict_kv!(dict_res, starlark_heap, "stdout", cmd_res.stdout, String);
    insert_dict_kv!(dict_res, starlark_heap, "stderr", cmd_res.stderr, String);
    insert_dict_kv!(dict_res, starlark_heap, "status", cmd_res.status, i32);

    Ok(dict_res)
}

fn handle_shell(cmd: String) -> Result<CommandOutput> {
    let command_string: &str;
    let command_args: Vec<&str>;

    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        command_string = "bash";
        command_args = ["-c", cmd.as_str()].to_vec();
    } else if cfg!(target_os = "windows") {
        command_string = "cmd";
        command_args = ["/c", cmd.as_str()].to_vec();
    } else {
        // linux and such
        command_string = "sh";
        command_args = ["-c", cmd.as_str()].to_vec();
    }

    let tmp_res = Command::new(command_string).args(command_args).output()?;
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&tmp_res.stdout).to_string(),
        stderr: String::from_utf8_lossy(&tmp_res.stderr).to_string(),
        status: tmp_res
            .status
            .code()
            .context("Failed to retrieve status code")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::{
        environment::{GlobalsBuilder, Module},
        eval::Evaluator,
        starlark_module,
        syntax::{AstModule, Dialect},
        values::Value,
    };

    #[test]
    fn test_sys_shell_current_user() -> anyhow::Result<()> {
        let res = handle_shell(String::from("whoami"))?.stdout.to_lowercase();
        println!("{}", res);
        assert!(
            res.contains("runner")
                || res.contains("administrator")
                || res.contains("root")
                || res.contains("user")
        );
        Ok(())
    }

    #[test]
    fn test_sys_shell_complex_linux() -> anyhow::Result<()> {
        if cfg!(target_os = "linux")
            || cfg!(target_os = "ios")
            || cfg!(target_os = "macos")
            || cfg!(target_os = "android")
            || cfg!(target_os = "freebsd")
            || cfg!(target_os = "openbsd")
            || cfg!(target_os = "netbsd")
        {
            let res = handle_shell(String::from(
                "cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'",
            ))?
            .stdout;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }
    #[test]
    fn test_sys_shell_complex_windows() -> anyhow::Result<()> {
        if cfg!(target_os = "windows") {
            let res =
                handle_shell(String::from("wmic useraccount get name | findstr /i admin"))?.stdout;
            assert!(
                res.contains("runner") || res.contains("Administrator") || res.contains("user")
            );
        }
        Ok(())
    }

    #[test]
    fn test_sys_shell_from_interpreter() -> anyhow::Result<()> {
        // Create test script
        let test_content = r#"
func_shell("whoami")
"#
        .to_string();

        // Setup starlark interpreter with handle to our function
        let ast = match AstModule::parse(
            "test.eldritch",
            test_content.to_owned(),
            &Dialect {
                enable_f_strings: true,
                ..Dialect::Extended
            },
        ) {
            Ok(res) => res,
            Err(err) => return Err(err.into_anyhow()),
        };

        #[starlark_module]
        #[allow(clippy::needless_lifetimes)]
        fn func_shell(builder: &mut GlobalsBuilder) {
            fn func_shell<'v>(starlark_heap: &'v Heap, cmd: String) -> anyhow::Result<Dict<'v>> {
                shell(starlark_heap, cmd)
            }
        }

        let globals = GlobalsBuilder::standard().with(func_shell).build();
        let module: Module = Module::new();

        let mut eval: Evaluator = Evaluator::new(&module);
        let res: Value = eval.eval_module(ast, &globals).unwrap();
        let res_string = res.to_string().to_lowercase();
        assert!(
            res_string.contains("runner")
                || res_string.contains("administrator")
                || res_string.contains("root")
                || res_string.contains("user")
        );
        Ok(())
    }
}
