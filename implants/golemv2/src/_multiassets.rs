use alloc::string::String;
use alloc::vec::Vec;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use eldritch_libassets::AssetsLibrary;
use eldritch_macros::eldritch_library_impl;
use alloc::borrow::Cow;
use crate::assetbackend::AssetBackend;
use anyhow;



/// A library that combines multiple AssetBackend implementations,
/// searching them in the order they were added.
#[eldritch_library_impl(AssetsLibrary)]
pub struct MultiAssetLibrary {
    // Stores a vector of boxed trait objects for runtime polymorphism.
    backends: Vec<Box<dyn AssetBackend>>,
    // Stores all asset names collected so far.
    asset_names: HashSet<String>,
}

impl core::fmt::Debug for MultiAssetLibrary {
fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MultiAssetLibrary")
            .finish()
    }
}

impl MultiAssetLibrary {
    /// Initializes an empty library.
    pub fn new() -> anyhow::Result<Self> {
        Ok(MultiAssetLibrary {
            backends: Vec::new(),
            asset_names: HashSet::new(),
        })
    }

    /// Adds an AssetBackend to the library.
    /// The order of addition determines the search precedence.
    /// Asset name shadowing is forbidden
    pub fn add<T>(&mut self, backend: T) -> anyhow::Result<()>
    where
        T: AssetBackend + 'static,
    {
        // Make a hashset of the new asset names
        let new_assets: HashSet<String> = backend.assets().into_iter()
            .map(Cow::into_owned)
            .collect();
        // See if any name overlap with existin assets
        let colliding_names: Vec<&str> = self.asset_names.intersection(&new_assets)
            .map(String::as_str)
            .collect();

        if colliding_names.len() > 0 {
            let error_message = format!(
                "Asset collision detected. The following asset names already exist in the library: {}",
                colliding_names.join(", ")
            );
            return Err(anyhow::Error::msg(error_message));
        };
        // Box the concrete type and store it as a trait object.
        self.asset_names.extend(new_assets);
        self.backends.push(Box::new(backend));
        Ok(())
    }

    // Get all the tomes from the asset locker
    pub fn tomes(&self) -> Vec<ParsedTome> {
        let mut tomes: Vec<ParsedTome> = Vec::new();
        let mut seen_files: HashSet<String> = HashSet::new(); // track names weve seen
        for library in &self.backends {
            for name_cow in library.assets() {
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
        for library in &self.backends {
            if let Some(file) = library.get(&name) {
                // Return immediately upon the first match
                return Ok(file.to_vec());
            }
        }
        Err(format!("asset not found: {}", name))
    }

    fn read(&self, name: String) -> Result<String, String> {
        let file = self.read_binary(name.clone())
            .map_err(|_| format!("asset not found: {}", name))?;

        String::from_utf8(file)
            .map_err(|e| format!("asset '{}' contains invalid UTF-8: {}", name, e))
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        let embedded_data = self.read_binary(src.clone())
            .map_err(|e| format!("copy failed: {}", e))?;

        let mut file = fs::File::create(&dest)
            .map_err(|e| format!("failed to create destination file '{}': {}", dest, e))?;

        file.write_all(&embedded_data)
            .map_err(|e| format!("failed to write data to destination file '{}': {}", dest, e))?;

        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.asset_names.iter().cloned().collect())
    }
}