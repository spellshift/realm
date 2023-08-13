use anyhow::Result;
use starlark::values::Heap;
use std::process::Command;
use std::str;

use eldritch_types::command_output::CommandOutput;

pub fn shell(cmd: String) -> Result<CommandOutput> {
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
        .output()
        .expect("failed to execute process");

    return Ok(CommandOutput{
        stdout: String::from_utf8(tmp_res.stdout)?,
        stderr: String::from_utf8(tmp_res.stderr)?,
        status: tmp_res.status.code().expect("Failed to retrieve error code"),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sys_shell_current_user() -> anyhow::Result<()>{
        let res = shell(String::from("whoami"))?.stdout;
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
            let res = shell(String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'"))?.stdout;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }
    #[test]
    fn test_sys_shell_complex_windows() -> anyhow::Result<()>{
        if cfg!(target_os = "windows") {
            let res = shell(String::from("wmic useraccount get name | findstr /i admin"))?.stdout;
            assert!(res.contains("runner") || res.contains("Administrator") || res.contains("user"));
        }
        Ok(())
    }
}