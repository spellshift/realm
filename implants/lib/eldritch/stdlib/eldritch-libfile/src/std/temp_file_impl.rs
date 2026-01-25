use alloc::string::String;
use alloc::string::ToString;

pub fn temp_file(name: Option<String>) -> Result<String, String> {
    let temp_dir = ::std::env::temp_dir();
    let file_name = name.unwrap_or_else(|| {
        // Simple random name generation if None
        use alloc::format;
        format!(
            "eldritch_{}",
            ::std::time::SystemTime::now()
                .duration_since(::std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    });
    let path = temp_dir.join(file_name);
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to convert temp path to string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_file() {
        let res = temp_file(None).unwrap();
        assert!(std::path::Path::new(&res).is_absolute());

        let res2 = temp_file(Some("foo.txt".to_string())).unwrap();
        assert!(res2.ends_with("foo.txt"));
    }
}
