// This file defines the necessary trait and macro to make standard
// RustEmbed structs compatible with the dynamic MultiAssetLibrary.
extern crate alloc;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt::Debug;
use rust_embed::EmbeddedFile;

// This trait is object-safe (`dyn AssetBackend`) because it is Sized, Send, Sync,
// and all methods return known, concrete types (EmbeddedFile or Vec).
pub trait AssetBackend: Debug + Send + Sync + 'static {
    fn get(&self, file_path: &str) -> Option<EmbeddedFile>;
    // Returns a concrete Vec for dynamic dispatch
    fn iter_items(&self) -> Vec<Cow<'static, str>>;
}

/// Automatically implements the object-safe AssetBackend trait
/// for a struct that uses the standard #[derive(rust_embed::RustEmbed)].
#[macro_export]
macro_rules! as_asset_backend {
    ($struct_name:ident) => {
        impl $crate::AssetBackend for $struct_name {
            fn get(&self, file_path: &str) -> Option<EmbeddedFile> {
                // Delegate to the derived standard RustEmbed::get (associated function)
                <Self as rust_embed::RustEmbed>::get(file_path)
            }

            fn iter_items(&self) -> Vec<Cow<'static, str>> {
                // Call the standard iter() and force collection into a concrete Vec
                <Self as rust_embed::RustEmbed>::iter().collect()
            }
        }
    };
}