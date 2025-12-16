use alloc::string::String;
use alloc::vec::Vec;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use eldritch_libassets::AssetsLibrary;
use eldritch_macros::eldritch_library_impl;

use crate::assetbackend::AssetBackend;

pub struct ParsedTome {
    pub name: String,
    pub eldritch: String,
}

/// A library that combines multiple AssetBackend implementations,
/// searching them in the order they were added.
#[eldritch_library_impl(AssetsLibrary)]
pub struct MultiAssetLibrary {
    // Stores a vector of boxed trait objects for runtime polymorphism.
    assets: Vec<Box<dyn AssetBackend>>,
}

impl core::fmt::Debug for MultiAssetLibrary {
fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Collect the Debug-printable items into a standard Vec
        let backends_formatted: Vec<_> = self.assets.iter().enumerate()
            .map(|(i, backend)| (i, backend)) // Create the (index, &Box<dyn AssetBackend>) tuple
            .collect();
        
        f.debug_struct("MultiAssetLibrary")
            .field("Backends", &backends_formatted)
            .finish()
    }
}

impl MultiAssetLibrary {
    /// Initializes an empty library.
    pub fn new() -> Self {
        MultiAssetLibrary { assets: Vec::new() }
    }

    /// Adds an AssetBackend to the library.
    /// The order of addition determines the search precedence.
    pub fn add<T>(&mut self, asset: T)
    where
        T: AssetBackend + 'static, // Must implement the trait and have 'static lifetime
    {
        // Box the concrete type and store it as a trait object.
        self.assets.push(Box::new(asset));
    }

    // Get all the tomes from the asset locker
    pub fn tomes(&self) -> Vec<ParsedTome> {
        let mut tomes: Vec<ParsedTome> = Vec::new();
        let mut seen_files: HashSet<String> = HashSet::new(); // track names weve seen
        for library in &self.assets {
            for name_cow in library.iter_items() {
                let file_path = name_cow.as_ref();
                // Only process files ending with the eldritch extensions
                if !file_path.ends_with("main.eldritch") && !file_path.ends_with("main.eldr") {
                    continue;
                }
                if !seen_files.insert(file_path.to_string()) {
                    continue; 
                }
                let eldritch_data = match library.as_ref().get(file_path) {
                    Some(data) => data,
                    None => continue, // Should not happen for a file listed in iter_items
                };
                let eldritch_content = match str::from_utf8(eldritch_data.as_ref()) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        eprintln!("Warning: Eldritch file '{}' has invalid UTF-8 and was skipped.", file_path);
                        continue;
                    }
                };

                tomes.push(ParsedTome {
                    name: file_path.to_string(), 
                    eldritch: eldritch_content,
                });
            }
        }
        tomes
    }
}

impl AssetsLibrary for MultiAssetLibrary {
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String> {
        // Iterate through the boxed trait objects (maintaining precedence order)
        for library in &self.assets {
            if let Some(file) = library.get(&name) {
                // Return immediately upon the first match
                return Ok(file.to_vec());
            }
        }
        Err(format!("Asset not found: {}", name))
    }

    fn read(&self, name: String) -> Result<String, String> {
        let file = self.read_binary(name.clone())
            .map_err(|_| format!("Asset not found: {}", name))?;

        String::from_utf8(file)
            .map_err(|e| format!("Asset '{}' contains invalid UTF-8: {}", name, e))
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        let embedded_data = self.read_binary(src.clone())
            .map_err(|e| format!("Copy failed: {}", e))?;

        let mut file = fs::File::create(&dest)
            .map_err(|e| format!("Failed to create destination file '{}': {}", dest, e))?;

        file.write_all(&embedded_data)
            .map_err(|e| format!("Failed to write data to destination file '{}': {}", dest, e))?;

        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        let mut all_assets = HashSet::new();

        // Iterate through all libraries and collect all asset names
        for library in &self.assets {
            for name in library.iter_items() {
                if name.ends_with("main.eldritch") || name.ends_with("main.eldr") ||
                   name.ends_with("metadata.yml") || name.ends_with("metadata.yaml") {
                    continue; // Skip eldritch files
                    }
                all_assets.insert(name);
            }
        }

        // Convert the set of unique Cow<'static, str> into a Vec<String>
        Ok(all_assets.into_iter().map(|c| c.into_owned()).collect())
    }
}