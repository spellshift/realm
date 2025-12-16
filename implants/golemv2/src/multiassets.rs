extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use eldritch_libassets::AssetsLibrary;
use eldritch_macros::eldritch_library_impl;

use crate::assetbackend::AssetBackend; 

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
                all_assets.insert(name);
            }
        }

        // Convert the set of unique Cow<'static, str> into a Vec<String>
        Ok(all_assets.into_iter().map(|c| c.into_owned()).collect())
    }
}