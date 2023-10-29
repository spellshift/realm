use anyhow::Result;
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
            Err(_error) => {
                "ERROR".to_string()
            }
        };
        println!("{host}");
        assert!(host != "ERROR");
        Ok(())
    }
}
