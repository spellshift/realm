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

    let separator = std::path::MAIN_SEPARATOR;
    let joined = alloc::format!("{}{}{}", base, separator, target);
    super::path_clean_impl::path_clean(joined)
}
