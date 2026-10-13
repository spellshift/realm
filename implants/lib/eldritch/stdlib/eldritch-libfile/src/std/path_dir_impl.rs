use alloc::string::String;

pub fn path_dir(path: String) -> Result<String, String> {
    if path.is_empty() {
        return Ok(".".into());
    }

    let separator = std::path::MAIN_SEPARATOR;
    let mut p = path.as_str();
    while p.len() > 1 && p.ends_with(separator) {
        p = &p[..p.len() - 1];
    }

    if p == separator.to_string() {
        return Ok(separator.to_string());
    }

    if let Some(idx) = p.rfind(separator) {
        if idx == 0 {
            Ok(separator.to_string())
        } else {
            let mut dir = &p[..idx];
            while dir.len() > 1 && dir.ends_with(separator) {
                dir = &dir[..dir.len() - 1];
            }
            Ok(dir.into())
        }
    } else {
        Ok(".".into())
    }
}
