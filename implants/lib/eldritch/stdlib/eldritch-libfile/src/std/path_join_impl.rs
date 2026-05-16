use alloc::string::String;

pub fn path_join(base: String, target: String) -> Result<String, String> {
    if base.is_empty() && target.is_empty() {
        return Ok("".into());
    }
    if base.is_empty() {
        return super::path_clean_impl::path_clean(target);
    }
    if target.is_empty() {
        return super::path_clean_impl::path_clean(base);
    }

    let joined = alloc::format!("{}/{}", base, target);
    super::path_clean_impl::path_clean(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_join() {
        assert_eq!(path_join("".into(), "".into()).unwrap(), "");
        assert_eq!(path_join("a".into(), "".into()).unwrap(), "a");
        assert_eq!(path_join("".into(), "b".into()).unwrap(), "b");
        assert_eq!(path_join("a".into(), "b".into()).unwrap(), "a/b");
        assert_eq!(path_join("a/".into(), "b".into()).unwrap(), "a/b");
        assert_eq!(path_join("a".into(), "/b".into()).unwrap(), "a/b");
        assert_eq!(path_join("a/".into(), "/b".into()).unwrap(), "a/b");
        assert_eq!(path_join("a/b".into(), "../c".into()).unwrap(), "a/c");
        assert_eq!(path_join("/a/b".into(), "../c".into()).unwrap(), "/a/c");
    }
}
