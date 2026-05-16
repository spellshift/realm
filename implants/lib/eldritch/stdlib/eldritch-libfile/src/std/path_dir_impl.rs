use alloc::string::String;

pub fn path_dir(path: String) -> Result<String, String> {
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
        if idx == 0 {
            Ok("/".into())
        } else {
            let mut dir = &p[..idx];
            while dir.len() > 1 && dir.ends_with('/') {
                dir = &dir[..dir.len() - 1];
            }
            Ok(dir.into())
        }
    } else {
        Ok(".".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_dir() {
        assert_eq!(path_dir("".into()).unwrap(), ".");
        assert_eq!(path_dir(".".into()).unwrap(), ".");
        assert_eq!(path_dir("/".into()).unwrap(), "/");
        assert_eq!(path_dir("//".into()).unwrap(), "/");
        assert_eq!(path_dir("abc".into()).unwrap(), ".");
        assert_eq!(path_dir("abc/def".into()).unwrap(), "abc");
        assert_eq!(path_dir("abc/def/".into()).unwrap(), "abc");
        assert_eq!(path_dir("abc/def//".into()).unwrap(), "abc");
        assert_eq!(path_dir("/abc".into()).unwrap(), "/");
        assert_eq!(path_dir("/abc/def".into()).unwrap(), "/abc");
        assert_eq!(path_dir("/abc/def/ghi".into()).unwrap(), "/abc/def");
    }
}
