use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use anyhow::{Context, Result};
use eldritch_core::Value;
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

struct CommandOutput {
    stdout: String,
    stderr: String,
    status: i32,
}

pub fn shell(cmd: String) -> Result<BTreeMap<String, Value>> {
    let cmd_res = handle_shell(cmd)?;

    let mut dict_res = BTreeMap::new();
    dict_res.insert("stdout".to_string(), Value::String(cmd_res.stdout));
    dict_res.insert("stderr".to_string(), Value::String(cmd_res.stderr));
    dict_res.insert("status".to_string(), Value::Int(cmd_res.status as i64));

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
    while unsafe { *ptr.offset(len) } != 0 {
        len += 1;
    }

    // Push it onto the list.
    let buf = unsafe { slice::from_raw_parts(ptr, len as usize) };
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

    #[test]
    fn test_sys_shell_current_user() -> anyhow::Result<()> {
        let expected = whoami::username();
        let res = handle_shell(String::from("whoami"))?.stdout;
        assert!(res.contains(&expected));
        Ok(())
    }
}
