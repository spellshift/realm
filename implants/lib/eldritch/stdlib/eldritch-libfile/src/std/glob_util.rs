#[cfg(feature = "stdlib")]
pub(crate) fn resolve_paths(
    path: &str,
) -> Result<alloc::vec::Vec<std::path::PathBuf>, alloc::string::String> {
    use alloc::format;
    use alloc::vec::Vec;

    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = Vec::new();
        for entry in glob::glob(path).map_err(|e| format!("Invalid glob pattern: {e}"))? {
            match entry {
                Ok(p) => paths.push(p),
                Err(e) => return Err(format!("Glob error: {e}")),
            }
        }
        if paths.is_empty() {
            return Err(format!("No files matched pattern: {path}"));
        }
        Ok(paths)
    } else {
        Ok(alloc::vec![std::path::PathBuf::from(path)])
    }
}

#[cfg(feature = "stdlib")]
pub(crate) fn resolve_first_path(path: &str) -> Result<std::path::PathBuf, alloc::string::String> {
    use alloc::format;
    let paths = resolve_paths(path)?;
    paths
        .into_iter()
        .next()
        .ok_or_else(|| format!("No files matched pattern: {path}"))
}
