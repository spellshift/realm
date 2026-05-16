use alloc::string::String;

pub fn path_separator() -> Result<String, String> {
    Ok(std::path::MAIN_SEPARATOR.to_string())
}
