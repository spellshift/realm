use anyhow::Result;
use std::println;
use whoami::hostname as whoHostname;

pub fn hostname() -> Result<String> {
    return Ok(whoHostname());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname() -> Result<()>{
        let host = match hostname() {
            Ok(tmp_hostname) => tmp_hostname,
            Err(error) => {
                return Err(anyhow::anyhow!("Unable to get system hostname{}", error));
            }
        };
        println!("{host}");
        Ok(())
    }
}