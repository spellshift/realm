// This file defines the necessary trait and macro to make standard
// RustEmbed structs compatible with the dynamic MultiAssetLibrary.
use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::anyhow;
use core::fmt::Debug;
use eldritch_libassets::std::AssetBackend;
use std::fs;
use std::path::PathBuf;

const MAX_RECURSION_DEPTH: usize = 10;

#[derive(Debug)]
pub struct DirectoryAssetBackend {
    root: PathBuf,
}

// An asset backend that reads from the local filesystem
impl DirectoryAssetBackend {
    /// Creates a new asset backend rooted at the given directory path.
    ///
    /// # Errors
    /// Returns a String error if the path does not exist or is not a directory.
    pub fn new(directory_name: &str) -> anyhow::Result<Self> {
        let root = PathBuf::from(directory_name);
        if !root.exists() {
            return Err(anyhow!("Directory not found: {}", directory_name));
        }
        if !root.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", directory_name));
        }

        // Canonicalize the path to resolve symlinks and '..' segments,
        // which helps in later path checks.
        let canonical_root = root
            .canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize path {}: {}", directory_name, e))?;

        Ok(DirectoryAssetBackend {
            root: canonical_root,
        })
    }

    /// Safely constructs the full, canonical path to the requested file.
    /// Returns `None` if the path attempts to leave the root directory.
    fn get_safe_path(&self, file_path: &str) -> Option<PathBuf> {
        // 1. Join the root and the relative file path.
        let mut path = self.root.clone();
        path.push(file_path);

        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            // If canonicalize fails (e.g., file doesn't exist), we treat it as not found.
            Err(_) => return None,
        };

        // Ensure the canonical path starts with the canonical root.
        // If it doesn't, the path contained '..' segments that successfully escaped the root.
        if !canonical_path.starts_with(&self.root) {
            // Path traversal attempt detected
            return None;
        }

        Some(canonical_path)
    }
}

// Implementing AssetBackend for DirectoryAssetBackend
impl AssetBackend for DirectoryAssetBackend {
    fn get(&self, file_path: &str) -> anyhow::Result<Vec<u8>> {
        let safe_path = self
            .get_safe_path(file_path)
            .ok_or(anyhow::anyhow!("invalid file path"))?;
        // The path is safe and exists. Read the contents.
        let data = fs::read(&safe_path)?;
        Ok(data)
    }

    fn assets(&self) -> Vec<Cow<'static, str>> {
        walkdir::WalkDir::new(&self.root)
            .max_depth(MAX_RECURSION_DEPTH)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                entry
                    .path()
                    .strip_prefix(&self.root)
                    .ok() // Get Ok(RelPath), discard Err
                    .and_then(|rel_path| rel_path.to_str()) // Get Some(str), discard None
                    .map(|s| Cow::Owned(s.to_string()))
            })
            .collect()
    }
}
