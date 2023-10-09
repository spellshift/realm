use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::Heap;
use starlark::values::dict::Dict;
use std::process::Command;
use std::str;

use super::CommandOutput;

pub fn shell(starlark_heap: &Heap, cmd: String) -> Result<Dict> {

    let cmd_res = handle_shell(cmd)?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    let stdout_value = starlark_heap.alloc_str(cmd_res.stdout.as_str());
    dict_res.insert_hashed(const_frozen_string!("stdout").to_value().get_hashed().unwrap(), stdout_value.to_value());

    let stderr_value = starlark_heap.alloc_str(cmd_res.stderr.as_str());
    dict_res.insert_hashed(const_frozen_string!("stderr").to_value().get_hashed().unwrap(), stderr_value.to_value());

    let status_value = starlark_heap.alloc(cmd_res.status);
    dict_res.insert_hashed(const_frozen_string!("status").to_value().get_hashed().unwrap(), status_value);

    Ok(dict_res)
}

fn handle_shell(cmd: String) -> Result<CommandOutput> {
    let command_string: &str;
    let command_args: Vec<&str>;

    if cfg!(target_os = "macos") {
        command_string = "bash";
        command_args = ["-c", cmd.as_str()].to_vec();
    } else if cfg!(target_os = "windows") {
        command_string = "cmd";
        command_args = ["/c", cmd.as_str()].to_vec();
    } else if cfg!(target_os = "linux") {
        command_string = "bash";
        command_args = ["-c", cmd.as_str()].to_vec();
    } else { // linux and such
        command_string = "bash";
        command_args = ["-c", cmd.as_str()].to_vec();
    }

    let tmp_res = Command::new(command_string)
        .args(command_args)
        .output()?;

    return Ok(CommandOutput{
        stdout: String::from_utf8(tmp_res.stdout)?,
        stderr: String::from_utf8(tmp_res.stderr)?,
        status: tmp_res.status.code().ok_or(anyhow::anyhow!("Failed to retrieve status code"))?,
    });
}

#[cfg(test)]
mod tests {
    use starlark::{syntax::{AstModule, Dialect}, starlark_module, environment::{GlobalsBuilder, Module}, eval::Evaluator, values::Value};

    use super::*;
    #[test]
    fn test_sys_shell_current_user() -> anyhow::Result<()>{
        let res = handle_shell(String::from("whoami"))?.stdout;
        println!("{}",res);
        assert!(res.contains("runner") || res.contains("Administrator") || res.contains("root") || res.contains("user"));
        Ok(())
    }

    #[test]
    fn test_sys_shell_complex_linux() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "macos") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = handle_shell(String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'"))?.stdout;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }
    #[test]
    fn test_sys_shell_complex_windows() -> anyhow::Result<()>{
        if cfg!(target_os = "windows") {
            let res = handle_shell(String::from("wmic useraccount get name | findstr /i admin"))?.stdout;
            assert!(res.contains("runner") || res.contains("Administrator") || res.contains("user"));
        }
        Ok(())
    }

    #[test]
    fn test_sys_shell_from_interpreter() -> anyhow::Result<()>{
        // Create test script
        let test_content = format!(r#"
func_shell("whoami")
"#);

        // Setup starlark interpreter with handle to our function
        let ast: AstModule;
        match AstModule::parse(
                "test.eldritch",
                test_content.to_owned(),
                &Dialect::Standard
            ) {
                Ok(res) => ast = res,
                Err(err) => return Err(err),
        }

        #[starlark_module]
        fn func_shell(builder: &mut GlobalsBuilder) {
            fn func_shell<'v>(starlark_heap: &'v Heap, cmd: String) -> anyhow::Result<Dict<'v>> {
                shell(starlark_heap, cmd)
            }
        }

        let globals = GlobalsBuilder::standard().with(func_shell).build();
        let module: Module = Module::new();

        let mut eval: Evaluator = Evaluator::new(&module);
        let res: Value = eval.eval_module(ast, &globals).unwrap();
        let res_string = res.to_string();
        assert!(res_string.contains("runner") || res_string.contains("Administrator") || res_string.contains("root") || res_string.contains("user"));
        Ok(())
    }
}