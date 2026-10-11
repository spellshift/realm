use alloc::string::String;

pub fn path_base(path: String) -> Result<String, String> {
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
        Ok(p[idx + 1..].into())
    } else {
        Ok(p.into())
    }
}
