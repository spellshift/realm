use anyhow::Result;
use std::process::{Command, exit};
use std::str;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use nix::{sys::wait::waitpid, unistd::{fork, ForkResult}};

// https://stackoverflow.com/questions/62978157/rust-how-to-spawn-child-process-that-continues-to-live-after-parent-receives-si#:~:text=You%20need%20to%20double%2Dfork,is%20not%20related%20to%20rust.&text=You%20must%20not%20forget%20to,will%20become%20a%20zombie%20process.

pub fn exec(path: String, args: Vec<String>, disown: Option<bool>) -> Result<String> {
    let should_disown = match disown {
        Some(disown_option) => disown_option,
        None => false,
    };
    
    if !should_disown {
        let res = Command::new(path)
            .args(args)
            .output()
            .expect("failed to execute process");
        
        let resstr = str::from_utf8(&res.stdout).unwrap();
        return Ok(String::from(resstr));
    }else{
        #[cfg(target_os = "windows")]
        return Err(anyhow::anyhow!("Windows is not supported for disowned processes."));

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        match unsafe{fork().expect("Failed to fork process")} {
            ForkResult::Parent { child } => {    
                // Wait for intermediate process to exit.
                waitpid(Some(child), None).unwrap();
                return Ok("No output".to_string());
            }
    
            ForkResult::Child => {
                match unsafe{fork().expect("Failed to fork process")} {
                    ForkResult::Parent { child } => {
                        if child.as_raw() < 0 { return Ok("Pid was negative. ERR".to_string()) }
                        exit(0)
                    }
            
                    ForkResult::Child => {
                        let _res = Command::new(path)
                            .args(args)
                            .output()
                            .expect("failed to execute process");
                        exit(0)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, time, thread, fs};

    use tempfile::NamedTempFile;

    use super::*;
    #[test]
    fn test_process_exec_current_user() -> anyhow::Result<()>{
        if cfg!(target_os = "linux") || 
        cfg!(target_os = "ios") || 
        cfg!(target_os = "android") || 
        cfg!(target_os = "freebsd") || 
        cfg!(target_os = "openbsd") ||
        cfg!(target_os = "netbsd") {
            let res = exec(String::from("/bin/sh"),vec![String::from("-c"), String::from("id -u")], Some(false))?;
            let mut bool_res = false; 
            if res == "1001\n" || res == "0\n" {
                bool_res = true;
            }
            assert_eq!(bool_res, true);
        }
        else if cfg!(target_os = "macos") {
            let res = exec(String::from("/bin/echo"),vec![String::from("hello")], Some(false))?;
            assert_eq!(res, "hello\n");
        }
        else if cfg!(target_os = "windows") {
            let res = exec(String::from("C:\\Windows\\System32\\cmd.exe"), vec![String::from("/c"), String::from("whoami")], Some(false))?;
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
            let res = exec(String::from("/bin/sh"), vec![String::from("-c"), String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'")], Some(false))?;
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
            let tmp_file = NamedTempFile::new()?;
            let path = String::from(tmp_file.path().to_str().unwrap());
            tmp_file.close()?;
    
            let _res = exec(String::from("/bin/sh"), vec![String::from("-c"), String::from(format!("touch {}", path.clone()))], Some(true))?;
            thread::sleep(time::Duration::from_secs(2));

            println!("{:?}", path.clone().as_str());
            assert!(Path::new(path.clone().as_str()).exists());

            let _ = fs::remove_file(path.as_str());
        }
        Ok(())
    }
    #[test]
    fn test_process_exec_complex_windows() -> anyhow::Result<()>{
        if cfg!(target_os = "windows") {
            let res = exec(String::from("C:\\Windows\\System32\\cmd.exe"), vec![String::from("/c"), String::from("wmic useraccount get name | findstr /i admin")], Some(false))?;
            assert_eq!(res.contains("runneradmin") || res.contains("Administrator"), true);
        }
        Ok(())
    }
}