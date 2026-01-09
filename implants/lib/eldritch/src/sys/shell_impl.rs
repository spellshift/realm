use super::super::insert_dict_kv;
use anyhow::{Context, Result};
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::dict::Dict;
use starlark::values::Heap;
use std::process::Command;
#[cfg(target_os = "windows")]
use {
    std::ffi::{OsStr, OsString},
    std::iter::once,
    std::os::windows::ffi::{OsStrExt, OsStringExt},
    std::path::Path,
    std::{slice, str},
    windows_sys::Win32::UI::Shell::CommandLineToArgvW,
};

use super::CommandOutput;

pub fn shell(starlark_heap: &'_ Heap, cmd: String) -> Result<Dict<'_>> {
    let cmd_res = handle_shell(cmd)?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    insert_dict_kv!(dict_res, starlark_heap, "stdout", cmd_res.stdout, String);
    insert_dict_kv!(dict_res, starlark_heap, "stderr", cmd_res.stderr, String);
    insert_dict_kv!(dict_res, starlark_heap, "status", cmd_res.status, i32);

    Ok(dict_res)
}

#[cfg(target_os = "windows")]
pub fn to_wstring(str: impl AsRef<Path>) -> Vec<u16> {
    OsStr::new(str.as_ref())
        .encode_wide()
        .chain(once(0))
        .collect()
}

#[cfg(target_os = "windows")]
pub unsafe fn os_string_from_wide_ptr(ptr: *const u16) -> OsString {
    let mut len = 0;
    while *ptr.offset(len) != 0 {
        len += 1;
    }

    // Push it onto the list.
    let buf = slice::from_raw_parts(ptr, len as usize);
    OsStringExt::from_wide(buf)
}

#[cfg(target_os = "windows")]
pub fn to_argv(command_line: &str) -> Vec<OsString> {
    let mut argv: Vec<OsString> = Vec::new();
    let mut argc = 0;
    unsafe {
        let args = CommandLineToArgvW(to_wstring(command_line).as_ptr(), &mut argc);

        for i in 0..argc {
            argv.push(os_string_from_wide_ptr(*args.offset(i as isize)));
        }

        // LocalFree shouldn't be needed this should get dropped
        // LocalFree(args as *const c_void);
    }
    argv
}

fn handle_shell(cmd: String) -> Result<CommandOutput> {
    #[cfg(not(target_os = "windows"))]
    {
        let command_string = "sh";
        let command_args = ["-c", cmd.as_str()].to_vec();
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

    #[cfg(target_os = "windows")]
    {
        let command_string = "cmd";
        let all_together = format!("/c {}", cmd);
        let new_arg = to_argv(all_together.as_str());
        let tmp_res = Command::new(command_string).args(new_arg).output()?;
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&tmp_res.stdout).to_string(),
            stderr: String::from_utf8_lossy(&tmp_res.stderr).to_string(),
            status: tmp_res
                .status
                .code()
                .context("Failed to retrieve status code")?,
        })
    }
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
        let expected = whoami::username().to_lowercase();
        let res = handle_shell(String::from("whoami"))?.stdout;
        assert!(res.contains(&expected));
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

    #[cfg(target_os = "windows")]
    #[test]
    fn test_sys_shell_complex_windows() -> anyhow::Result<()> {
        let res = handle_shell(String::from("echo admin | findstr /i admin"))?.stdout;
        assert!(res.contains("admin"));
        Ok(())
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_sys_shell_argv_parse() -> anyhow::Result<()> {
        let cmd = r#"cmd.exe /c cmd.exe /c cmd.exe /c dir "C:\Program Files\Windows Defender""#;
        let res = to_argv(cmd);
        assert_eq!(res.len(), 8);
        assert_eq!(res[0], OsString::from("cmd.exe"));
        assert_eq!(res[7], OsString::from(r"C:\Program Files\Windows Defender"));
        let cmd = r#"cmd.exe /c reg query "HKLM\Software\Microsoft\Windows NT\CurrentVersion\Winlogon" /v LegalNoticeCaption"#;
        let res = to_argv(cmd);
        assert_eq!(res.len(), 7);
        assert_eq!(res[0], OsString::from("cmd.exe"));
        assert_eq!(
            res[4],
            OsString::from(r"HKLM\Software\Microsoft\Windows NT\CurrentVersion\Winlogon")
        );
        let cmd = r#"/c reg query "HKLM\Software\Microsoft\Windows NT\CurrentVersion\Winlogon" /v LegalNoticeCaption"#;
        let res = to_argv(cmd);
        assert_eq!(res.len(), 6);
        assert_eq!(res[0], OsString::from("/c"));
        assert_eq!(
            res[3],
            OsString::from(r"HKLM\Software\Microsoft\Windows NT\CurrentVersion\Winlogon")
        );
        Ok(())
    }

    #[test]
    fn test_sys_shell_spaces_windows() -> anyhow::Result<()> {
        if cfg!(target_os = "windows") {
            let res = handle_shell(String::from(
                r#"cmd.exe /c dir "C:\Program Files\Windows Defender""#,
            ))?;
            assert!(res.stdout.contains("MsMpEng.exe"));
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
        fn func_shell(_builder: &mut GlobalsBuilder) {
            fn func_shell<'v>(starlark_heap: &'v Heap, cmd: String) -> anyhow::Result<Dict<'v>> {
                shell(starlark_heap, cmd)
            }
        }

        let globals = GlobalsBuilder::standard().with(func_shell).build();
        let module: Module = Module::new();

        let mut eval: Evaluator = Evaluator::new(&module);
        let res: Value = eval.eval_module(ast, &globals).unwrap();
        let res_string = res.to_str();
        let expected = whoami::username().to_lowercase();
        assert!(res_string.contains(&expected));
        Ok(())
    }
}
