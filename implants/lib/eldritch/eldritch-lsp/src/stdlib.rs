use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Manages an index of the Eldritch Standard Library.
///
/// This index allows the LSP to be aware of available standard library modules
/// and functions, enabling context-aware autocompletion even when files are
/// not explicitly imported in the current workspace.
#[derive(Debug)]
pub struct StdlibIndex {
    /// A set of known module names (e.g., "http", "file", "sys").
    /// In a real implementation, this might map names to function signatures.
    pub modules: HashSet<String>,
}

impl StdlibIndex {
    pub fn new() -> Self {
        Self {
            modules: HashSet::new(),
        }
    }

    /// Recursively scans the given root path for Eldritch standard library modules.
    ///
    /// It looks for files with `.eldritch` extension or directories that imply module structure.
    /// For this simplified implementation, we assume directories in `stdlib/` represent modules
    /// (e.g. `stdlib/eldritch-libhttp` -> "http").
    pub fn scan(&mut self, root_path: PathBuf) {
        log::info!("Scanning stdlib at {:?}", root_path);

        for entry in WalkDir::new(&root_path)
            .min_depth(1)
            .max_depth(2) // optimize: stdlib structure is shallow
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("eldritch-lib") {
                        let module_name = dir_name.trim_start_matches("eldritch-lib");
                        if !module_name.is_empty() {
                            self.modules.insert(module_name.to_string());
                            log::debug!("Found stdlib module: {}", module_name);
                        }
                    }
                }
            }
        }

        // Add hardcoded core modules if scanning fails or structure is different
        if self.modules.is_empty() {
             log::warn!("Stdlib scan yielded no results. Using fallback core modules.");
             let defaults = vec!["agent", "assets", "crypto", "file", "http", "pivot", "process", "random", "regex", "report", "sys", "time"];
             for m in defaults {
                 self.modules.insert(m.to_string());
             }
        }
    }

    /// Returns a list of all known module names for autocompletion.
    pub fn get_completions(&self) -> Vec<String> {
        let mut completions: Vec<String> = self.modules.iter().cloned().collect();
        completions.sort();
        completions
    }
}
