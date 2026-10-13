use alloc::string::String;
use alloc::vec::Vec;

pub fn path_clean(path: String) -> Result<String, String> {
    if path.is_empty() {
        return Ok(".".into());
    }

    let separator = std::path::MAIN_SEPARATOR;
    let is_absolute = path.starts_with(separator);
    let mut out: Vec<&str> = Vec::new();

    let segments = path.split(separator);

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
        return Ok(separator.to_string());
    }

    if out.is_empty() {
        return Ok(".".into());
    }

    let joined = out.join(&separator.to_string());
    if is_absolute {
        Ok(alloc::format!("{}{}", separator, joined))
    } else {
        Ok(joined)
    }
}
