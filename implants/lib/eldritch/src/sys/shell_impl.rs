use anyhow::Result;
use std::process::Command;
use std::str;

pub fn shell(cmd: String) -> Result<String> {
    if cfg!(target_os = "linux") || 
            cfg!(target_os = "ios") || 
            cfg!(target_os = "android") || 
            cfg!(target_os = "freebsd") || 
            cfg!(target_os = "openbsd") ||
            cfg!(target_os = "netbsd") {

        let res = Command::new("bash")
            .args(["-c", cmd.as_str()])
            .output()
            .expect("failed to execute process");
        let resstr = str::from_utf8(&res.stdout).unwrap();
        return Ok(String::from(resstr));
    }
    else if cfg!(target_os = "macos") {
        let res = Command::new("bash")
            .args(["-c", cmd.as_str()])
            .output()
            .expect("failed to execute process");
        let resstr = str::from_utf8(&res.stdout).unwrap();
        return Ok(String::from(resstr));
    }
    else if cfg!(target_os = "windows") {
        let res = Command::new("cmd")
            .args(["/C", cmd.as_str()])
            .output()
            .expect("failed to execute process");
        let resstr = str::from_utf8(&res.stdout).unwrap();
        return Ok(String::from(resstr));
    }else{
        return Err(anyhow::anyhow!("This OS isn't supported by sys.shell.\n\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_process_shell_current_user() -> anyhow::Result<()>{
        let res = shell(String::from("whoami"))?;
        println!("{:?}", res);
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let mut bool_res = false;
            if res == "runner\n" || res == "root\n" {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        else if cfg!(target_os = "macos") {
            let mut bool_res = false;
            if res == "runner\n" || res == "root\n" {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        else if cfg!(target_os = "windows") {
            let mut bool_res = false;
            if res.contains("runneradmin") || res.contains("Administrator") {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        Ok(())
    }
    #[test]
    fn test_process_shell_complex_linux() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "macos") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = shell(String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'"))?;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }
    #[test]
    fn test_process_shell_complex_windows() -> anyhow::Result<()>{
        if cfg!(target_os = "windows") {
            let res = shell(String::from("wmic useraccount get name | findstr /i admin"))?;
            assert_eq!(res.contains("runneradmin") || res.contains("Administrator"), true);
        }
        Ok(())
    }
}