use alloc::string::String;

pub fn path_base(path: String) -> Result<String, String> {
    if path.is_empty() {
        return Ok(".".into());
    }

    let mut p = path.as_str();
    while p.len() > 1 && p.ends_with('/') {
        p = &p[..p.len() - 1];
    }

    if p == "/" {
        return Ok("/".into());
    }

    if let Some(idx) = p.rfind('/') {
        Ok(p[idx + 1..].into())
    } else {
        Ok(p.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_base() {
        assert_eq!(path_base("".into()).unwrap(), ".");
        assert_eq!(path_base("/".into()).unwrap(), "/");
        assert_eq!(path_base("//".into()).unwrap(), "/");
        assert_eq!(path_base("abc".into()).unwrap(), "abc");
        assert_eq!(path_base("abc/def".into()).unwrap(), "def");
        assert_eq!(path_base("abc/def/".into()).unwrap(), "def");
        assert_eq!(path_base("abc/def//".into()).unwrap(), "def");
        assert_eq!(path_base("/abc".into()).unwrap(), "abc");
        assert_eq!(path_base("/abc/def".into()).unwrap(), "def");
        assert_eq!(path_base(".".into()).unwrap(), ".");
        assert_eq!(path_base("..".into()).unwrap(), "..");
    }
}
