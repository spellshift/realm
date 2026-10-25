pub fn whereis(bin: String) -> Result<alloc::string::String, alloc::string::String> {
    if let Ok(path) = which::which(bin) {
        return Ok(path.to_string_lossy().into_owned());
    }

    Ok("".to_string())
}