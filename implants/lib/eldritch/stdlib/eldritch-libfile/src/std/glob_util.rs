#[cfg(feature = "stdlib")]
pub(crate) fn resolve_paths(
    path: &str,
    skip_dirs: bool,
) -> Result<alloc::vec::Vec<std::path::PathBuf>, alloc::string::String> {
    use alloc::format;
    use alloc::vec::Vec;

    if path.contains('*') || path.contains('?') || path.contains('[') {
        let mut paths = Vec::new();
        for entry in glob::glob(path).map_err(|e| format!("Invalid glob pattern: {e}"))? {
            match entry {
                Ok(p) => {
                    if skip_dirs && p.is_dir() {
                        continue;
                    }
                    paths.push(p);
                }
                Err(e) => return Err(format!("Glob error: {e}")),
            }
        }
        if paths.is_empty() {
            return Err(format!("No files matched pattern: {path}"));
        }
        Ok(paths)
    } else {
        let p = std::path::PathBuf::from(path);
        if skip_dirs && p.is_dir() {
            return Err(format!("path '{}' is a directory", path));
        }
        Ok(alloc::vec![p])
    }
}

#[cfg(feature = "stdlib")]
pub(crate) fn resolve_first_path(
    path: &str,
    skip_dirs: bool,
) -> Result<std::path::PathBuf, alloc::string::String> {
    use alloc::format;
    let paths = resolve_paths(path, skip_dirs)?;
    paths
        .into_iter()
        .next()
        .ok_or_else(|| format!("No files matched pattern: {path}"))
}
