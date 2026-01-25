use alloc::string::String;
use alloc::string::ToString;

pub fn pwd() -> Result<Option<String>, String> {
    Ok(::std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd() {
        let res = pwd().unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(std::path::Path::new(&res).is_absolute());
    }
}
