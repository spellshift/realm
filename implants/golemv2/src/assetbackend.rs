// This file defines the necessary trait and macro to make standard
// RustEmbed structs compatible with the dynamic MultiAssetLibrary.
extern crate alloc;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt::Debug;
use rust_embed::{EmbeddedFile, Metadata};
use std::fs;
use std::path::{Path, PathBuf};


// This trait is object-safe (`dyn AssetBackend`) because it is Sized, Send, Sync,
// and all methods return known, concrete types (EmbeddedFile or Vec).
pub trait AssetBackend: Debug + Send + Sync + 'static {
    fn backend_type(&self) -> &'static str;
    fn backend_source(&self) -> String;
    fn get(&self, file_path: &str) -> Option<Cow<'static, [u8]>>;
    // Returns a concrete Vec for dynamic dispatch
    fn iter_items(&self) -> Vec<Cow<'static, str>>;
}

/// Automatically implements the object-safe AssetBackend trait
/// for a struct that uses the standard #[derive(rust_embed::RustEmbed)].
#[macro_export]
macro_rules! asset_backend_embedded {
    // This macro takes the struct name ($name) and the folder path ($folder_path)
    ($name:ident, $folder_path:literal) => {
        // A. Define the struct with #[derive(RustEmbed)]
        #[derive(Debug, rust_embed::RustEmbed)]
        #[folder = $folder_path]
        pub struct $name;

        impl $name {
            fn folder_name() -> &'static str {
                $folder_path
            }
        }
        
        impl $crate::assetbackend::AssetBackend for $name {

            fn backend_type(&self) -> &'static str {
                "embedded"
            }



            fn backend_source(&self) -> String {
                Self::folder_name().to_string() 
            }

            fn get(&self, file_path: &str) -> Option<Cow<'static, [u8]>> {
                let embedded_file: rust_embed::EmbeddedFile = <Self as rust_embed::RustEmbed>::get(file_path)?;
                Some(embedded_file.data)
            }

            fn iter_items(&self) -> Vec<Cow<'static, str>> {
                // Call the standard iter() and force collection into a concrete Vec
                <Self as rust_embed::RustEmbed>::iter().collect()
            }
        }
    };
}

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
    pub fn new(directory_name: &str) -> Result<Self, String> {
        let root = PathBuf::from(directory_name);
        if !root.exists() {
            return Err(format!("Directory not found: {}", directory_name));
        }
        if !root.is_dir() {
            return Err(format!("Path is not a directory: {}", directory_name));
        }
        
        // Canonicalize the path to resolve symlinks and '..' segments,
        // which helps in later path checks.
        let canonical_root = root.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path {}: {}", directory_name, e))?;

        Ok(DirectoryAssetBackend { root: canonical_root })
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
    fn backend_source(&self) -> std::string::String {
        self.root.to_string_lossy().into_owned()
    }

    fn backend_type(&self) -> &'static str {
        "dir"    
    }

    fn get(&self, file_path: &str) -> Option<Cow<'static, [u8]>> {
        let safe_path = self.get_safe_path(file_path)?;
        // The path is safe and exists. Read the contents.
        let data = fs::read(&safe_path).ok()?;
        Some(Cow::Owned(data))
    }

    fn iter_items(&self) -> Vec<Cow<'static, str>> {
        walkdir::WalkDir::new(&self.root)
        .max_depth(MAX_RECURSION_DEPTH) 
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            entry.path()
                 .strip_prefix(&self.root).ok() // Get Ok(RelPath), discard Err
                 .and_then(|rel_path| rel_path.to_str()) // Get Some(str), discard None
                 .map(|s| Cow::Owned(s.to_string()))
        })
        .collect()
    }
}