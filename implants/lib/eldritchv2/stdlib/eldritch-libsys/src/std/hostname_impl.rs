use anyhow::Result;

pub fn hostname() -> Result<String> {
    Ok(whoami::fallible::hostname()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname() -> Result<()> {
        let host = match hostname() {
            Ok(tmp_hostname) => tmp_hostname,
            Err(_error) => "ERROR".to_string(),
        };
        // Use println! or equivalent if std is available, but for test logic assertion matters more.
        assert!(host != "ERROR");
        Ok(())
    }
}
