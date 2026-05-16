use alloc::string::String;
use alloc::string::ToString;

pub fn path_abs(path: String) -> Result<String, String> {
    if path.is_empty() {
        return Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned());
    }

    let p = std::path::Path::new(&path);
    if p.is_absolute() {
        return super::path_clean_impl::path_clean(path);
    }

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let joined = cwd.join(p);
    super::path_clean_impl::path_clean(joined.to_string_lossy().into_owned())
}
