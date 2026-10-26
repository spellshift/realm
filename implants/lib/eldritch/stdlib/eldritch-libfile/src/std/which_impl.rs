use alloc::string::{String, ToString};
use anyhow::{Context, Result};
use which::which as which_fn;

pub fn which(bin: String) -> Result<String, String> {
    match which_fn(bin).context("Failed to find binary") {
        Ok(res) => match res.into_os_string().into_string() {
            Ok(res_str) => Ok(res_str),
            Err(err) => Err(format!("Failed to convert path to string: {err:?}")),
        },
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_which() -> anyhow::Result<()> {
        let mut path_str = env::var("PATH").unwrap_or_default();

        #[cfg(target_os = "windows")]
        let current_bin_dir = "C:\\Windows\\System32\\";
        #[cfg(target_os = "windows")]
        if !path_str.contains(current_bin_dir) {
            path_str = format!("{current_bin_dir};{path_str}");
            unsafe { env::set_var("PATH", path_str) };
        }
        #[cfg(not(target_os = "windows"))]
        let current_bin_dir = "/bin";
        #[cfg(not(target_os = "windows"))]
        if !path_str.contains(current_bin_dir) {
            path_str = format!("{current_bin_dir}:{path_str}");
            unsafe { env::set_var("PATH", path_str) };
        }

        #[cfg(target_os = "windows")]
        let res = which(String::from("cmd")).unwrap();
        #[cfg(target_os = "windows")]
        assert!(res.to_lowercase().contains("cmd.exe"));

        #[cfg(not(target_os = "windows"))]
        let res = which(String::from("sh")).unwrap();
        #[cfg(not(target_os = "windows"))]
        assert!(res.contains("/sh"));

        Ok(())
    }
}
