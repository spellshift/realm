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
    std::os::windows::process::CommandExt,
    std::path::Path,
    std::{slice, str},
    windows_sys::Win32::UI::Shell::CommandLineToArgvW,
};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
const STARTF_USESTDHANDLES: u32 = 0x00000100;

#[cfg(target_os = "windows")]
const SHELL_WAIT_TIMEOUT_MS: u32 = 30_000;

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
        // if impersonation token stored, use CreateProcessWithTokenW
        // Command::new uses process token, not thread token
        if let Some(output) = try_shell_with_token(&cmd) {
            return output;
        }

        let command_string = "cmd";
        let all_together = format!("/c {}", cmd);
        let new_arg = to_argv(all_together.as_str());
        let tmp_res = Command::new(command_string)
            .args(new_arg)
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;
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

// spawn cmd.exe using CreateProcessWithTokenW if there is an impersonation token active
#[cfg(target_os = "windows")]
fn try_shell_with_token(cmd: &str) -> Option<Result<CommandOutput>> {
    use super::tokens_impl::get_active_token_handle;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        DuplicateTokenEx, SecurityImpersonation, TOKEN_ALL_ACCESS, TokenPrimary,
    };
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::Threading::{
        CreateProcessWithTokenW, PROCESS_INFORMATION, STARTUPINFOW, WaitForSingleObject,
    };

    let imp_token = get_active_token_handle()?;

    // check SeImpersonatePrivilege on process token before calling CreateProcessWithTokenW
    {
        use super::tokens_impl::check_privilege;
        if !matches!(check_privilege("SeImpersonatePrivilege"), Ok(true)) {
            // fall back to normal Command::new
            return None;
        }
    }

    // dup as primary token (CreateProcessWithTokenW needs primary)
    let mut primary_token = std::ptr::null_mut();
    if unsafe {
        DuplicateTokenEx(
            imp_token as *mut std::ffi::c_void,
            TOKEN_ALL_ACCESS,
            std::ptr::null(),
            SecurityImpersonation,
            TokenPrimary,
            &mut primary_token,
        )
    } == 0
    {
        return Some(Err(anyhow::anyhow!(
            "DuplicateTokenEx to primary failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    // create pipes for stdout capture
    let mut stdout_read = std::ptr::null_mut();
    let mut stdout_write = std::ptr::null_mut();
    let mut sa: windows_sys::Win32::Security::SECURITY_ATTRIBUTES = unsafe { std::mem::zeroed() };
    sa.nLength = std::mem::size_of::<windows_sys::Win32::Security::SECURITY_ATTRIBUTES>() as u32;
    sa.bInheritHandle = 1;

    if unsafe { CreatePipe(&mut stdout_read, &mut stdout_write, &sa, 0) } == 0 {
        unsafe { CloseHandle(primary_token) };
        return Some(Err(anyhow::anyhow!(
            "CreatePipe failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    let cmdline = format!("cmd /c {}", cmd);
    let mut cmdline_wide: Vec<u16> = cmdline.encode_utf16().chain(std::iter::once(0)).collect();

    // target interactive desktop so child can init DLLs across sessions
    let mut desktop_wide: Vec<u16> = "WinSta0\\Default\0".encode_utf16().collect();

    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    si.lpDesktop = desktop_wide.as_mut_ptr();
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = stdout_write;
    si.hStdError = stdout_write;

    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    // temporarily revert impersonation so CreateProcessWithTokenW runs under the process token
    // SYSTEM impersonation tokens can cause access denied in shell due to session 0/1 mismatch
    unsafe { windows_sys::Win32::Security::RevertToSelf() };

    // build env block for token user (for some reason LOGON_WITH_PROFILE explodes when impersonating from SYSTEM -> Administrator)
    let mut env_block: *mut std::ffi::c_void = std::ptr::null_mut();
    let has_env = unsafe {
        windows_sys::Win32::System::Environment::CreateEnvironmentBlock(
            &mut env_block,
            primary_token,
            0,
        )
    } != 0;

    const CREATE_UNICODE_ENVIRONMENT: u32 = 0x00000400;
    let flags = CREATE_NO_WINDOW
        | if has_env {
            CREATE_UNICODE_ENVIRONMENT
        } else {
            0
        };

    let ok = unsafe {
        CreateProcessWithTokenW(
            primary_token,
            0,
            std::ptr::null(),
            cmdline_wide.as_mut_ptr(),
            flags,
            if has_env { env_block } else { std::ptr::null() },
            std::ptr::null(),
            &si,
            &mut pi,
        )
    };

    if has_env {
        unsafe { windows_sys::Win32::System::Environment::DestroyEnvironmentBlock(env_block) };
    }

    // re-apply impersonation token on this thread
    unsafe {
        windows_sys::Win32::Security::ImpersonateLoggedOnUser(imp_token as *mut std::ffi::c_void);
    }

    unsafe { CloseHandle(primary_token) };
    unsafe { CloseHandle(stdout_write) };

    if ok == 0 {
        unsafe { CloseHandle(stdout_read) };
        return Some(Err(anyhow::anyhow!(
            "CreateProcessWithTokenW failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    unsafe { WaitForSingleObject(pi.hProcess, SHELL_WAIT_TIMEOUT_MS) };

    let mut exit_code: u32 = 0;
    unsafe {
        windows_sys::Win32::System::Threading::GetExitCodeProcess(pi.hProcess, &mut exit_code);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    };

    let mut output = Vec::new();
    {
        use std::io::Read;
        use std::os::windows::io::FromRawHandle;
        let mut reader =
            unsafe { std::fs::File::from_raw_handle(stdout_read as *mut std::ffi::c_void) };
        let _ = reader.read_to_end(&mut output);
    }

    Some(Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output).to_string(),
        stderr: "".to_string(),
        status: exit_code as i32,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_shell_current_user() -> anyhow::Result<()> {
        let expected = whoami::username().to_lowercase();
        let res = handle_shell(String::from("whoami"))?.stdout;
        assert!(res.contains(&expected));
        Ok(())
    }

    // uses own token to exercise CreateProcessWithTokenW path
    #[cfg(target_os = "windows")]
    #[test]
    fn test_sys_shell_with_token() -> anyhow::Result<()> {
        use super::super::tokens_impl::{TOKEN_STORE, store_token};

        let mut token_handle = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                windows_sys::Win32::System::Threading::OpenProcessToken(
                    windows_sys::Win32::System::Threading::GetCurrentProcess(),
                    windows_sys::Win32::Security::TOKEN_DUPLICATE
                        | windows_sys::Win32::Security::TOKEN_QUERY
                        | windows_sys::Win32::Security::TOKEN_IMPERSONATE,
                    &mut token_handle,
                )
            },
            0,
            "OpenProcessToken failed"
        );

        // duplicate so we have a separate handle
        let mut dup_token = std::ptr::null_mut();
        assert_ne!(
            unsafe {
                windows_sys::Win32::Security::DuplicateTokenEx(
                    token_handle,
                    windows_sys::Win32::Security::TOKEN_DUPLICATE
                        | windows_sys::Win32::Security::TOKEN_QUERY
                        | windows_sys::Win32::Security::TOKEN_IMPERSONATE,
                    std::ptr::null(),
                    windows_sys::Win32::Security::SecurityImpersonation,
                    windows_sys::Win32::Security::TokenImpersonation,
                    &mut dup_token,
                )
            },
            0,
            "DuplicateTokenEx failed"
        );
        unsafe { windows_sys::Win32::Foundation::CloseHandle(token_handle) };

        let id = store_token(dup_token as isize, "test:self".to_string());

        unsafe {
            windows_sys::Win32::Security::ImpersonateLoggedOnUser(dup_token);
        }

        // shell should use CreateProcessWithTokenW path
        let res = handle_shell("whoami".to_string())?;
        let expected = whoami::username().to_lowercase();
        assert!(
            res.stdout.to_lowercase().contains(&expected),
            "Expected '{}' in stdout: '{}'",
            expected,
            res.stdout
        );

        unsafe { windows_sys::Win32::Security::RevertToSelf() };
        if let Ok(mut store) = TOKEN_STORE.lock() {
            for entry in store.iter_mut() {
                entry.active = false;
            }
        }

        Ok(())
    }
}
