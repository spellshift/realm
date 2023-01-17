use anyhow::Result;
use std::process::{Command, exit, id};
use std::str;
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};

// https://stackoverflow.com/questions/62978157/rust-how-to-spawn-child-process-that-continues-to-live-after-parent-receives-si#:~:text=You%20need%20to%20double%2Dfork,is%20not%20related%20to%20rust.&text=You%20must%20not%20forget%20to,will%20become%20a%20zombie%20process.

pub fn exec(path: String, args: Vec<String>, disown: bool) -> Result<String> {
    if !disown || cfg!(target_os = "windows") {
        let res = Command::new(path)
            .args(args)
            .output()
            .expect("failed to execute process");
        
        let resstr = str::from_utf8(&res.stdout).unwrap();
        return Ok(String::from(resstr));
    }else{
        match unsafe{fork().expect("Failed to fork process")} {
            ForkResult::Parent { child } => {
                println!("Try to kill me to check if the target process will be killed");
    
                // Wait for intermediate process to exit.
                waitpid(Some(child), None).unwrap();
    
                return Ok(format!("PID: {}\nchild PID: {}", id(), child.as_raw()).to_string());
            }
    
            ForkResult::Child => {
                match unsafe{fork().expect("Failed to fork process")} {
                    ForkResult::Parent { child } => {
                        return Ok(format!("Background process started {}", child.as_raw()))
                    }
            
                    ForkResult::Child => {
                        let _res = Command::new(path)
                            .args(args)
                            .output()
                            .expect("failed to execute process");
                        return Ok("exit".to_string());
                    }
                }
                // Kill ourselves after spawning the new process
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    #[test]
    fn test_process_exec_current_user() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = exec(String::from("/bin/sh"),vec![String::from("-c"), String::from("id -u")], false)?;
            println!("{:?}", res);
            let mut bool_res = false; 
            if res == "1000\n" || res == "0\n" {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        else if cfg!(target_os = "macos") {
            let res = exec(String::from("/bin/zsh"),vec![String::from("id"), String::from("-u")], false)?;
            let mut bool_res = false;
            if res == "501\n" || res == "0\n" {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        else if cfg!(target_os = "windows") {
            let res = exec(String::from("C:\\Windows\\System32\\cmd.exe"), vec![String::from("/c"), String::from("whoami")], false)?;
            let mut bool_res = false;
            if res.contains("runneradmin") || res.contains("Administrator") {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        Ok(())
    }
    #[test]
    fn test_process_exec_complex_linux() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "macos") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = exec(String::from("/bin/sh"), vec![String::from("-c"), String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'")], false)?;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }

    // This is a manual test:
    // Example results:
    // 42284 pts/0    S      0:00 /workspaces/realm/implants/target/debug/deps/eldritch-a23fc08ee1443dc3 test_process_exec_disown_linux --nocapture
    // 42285 pts/0    S      0:00  \_ /bin/sh -c sleep 600
    // 42286 pts/0    S      0:00      \_ sleep 600
    #[test]
    fn test_process_exec_disown_linux() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "macos") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = exec(String::from("/bin/sh"), vec![String::from("-c"), String::from("sleep 600")], true)?;
            println!("{}", res);
            let rs = Path::new("/tmp/win").exists();
            assert_eq!(rs, true);
        }
        Ok(())
    }
    #[test]
    fn test_process_exec_complex_windows() -> anyhow::Result<()>{
        if cfg!(target_os = "windows") {
            let res = exec(String::from("C:\\Windows\\System32\\cmd.exe"), vec![String::from("/c"), String::from("wmic useraccount get name | findstr /i admin")], false)?;
            assert_eq!(res.contains("runneradmin") || res.contains("Administrator"), true);
        }
        Ok(())
    }
}