use alloc::string::String;
use alloc::vec::Vec;

pub fn path_clean(path: String) -> Result<String, String> {
    if path.is_empty() {
        return Ok(".".into());
    }

    let is_absolute = path.starts_with('/');
    let mut out: Vec<&str> = Vec::new();

    let segments = path.split('/');

    for segment in segments {
        match segment {
            "" | "." => continue,
            ".." => {
                if let Some(last) = out.last() {
                    if *last != ".." {
                        out.pop();
                        continue;
                    }
                }
                if !is_absolute {
                    out.push("..");
                }
            }
            _ => out.push(segment),
        }
    }

    if out.is_empty() && is_absolute {
        return Ok("/".into());
    }

    if out.is_empty() {
        return Ok(".".into());
    }

    let joined = out.join("/");
    if is_absolute {
        Ok(alloc::format!("/{}", joined))
    } else {
        Ok(joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_clean() {
        assert_eq!(path_clean("".into()).unwrap(), ".");
        assert_eq!(path_clean("abc".into()).unwrap(), "abc");
        assert_eq!(path_clean("abc/def".into()).unwrap(), "abc/def");
        assert_eq!(path_clean("a/b/../c".into()).unwrap(), "a/c");
        assert_eq!(path_clean("a/b/../../c".into()).unwrap(), "c");
        assert_eq!(path_clean("a/b/../../../c".into()).unwrap(), "../c");
        assert_eq!(path_clean("/a/b/../../../c".into()).unwrap(), "/c");
        assert_eq!(path_clean("a//b//c".into()).unwrap(), "a/b/c");
        assert_eq!(path_clean("./a/b".into()).unwrap(), "a/b");
        assert_eq!(path_clean("a/./b".into()).unwrap(), "a/b");
        assert_eq!(path_clean("/a/./b".into()).unwrap(), "/a/b");
        assert_eq!(path_clean("/".into()).unwrap(), "/");
        assert_eq!(path_clean("//".into()).unwrap(), "/");
        assert_eq!(path_clean("///".into()).unwrap(), "/");
        assert_eq!(path_clean(".".into()).unwrap(), ".");
        assert_eq!(path_clean("..".into()).unwrap(), "..");
        assert_eq!(path_clean("../..".into()).unwrap(), "../..");
    }
}
