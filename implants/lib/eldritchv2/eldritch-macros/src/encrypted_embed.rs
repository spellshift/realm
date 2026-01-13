use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use flate2::Compression;
use flate2::write::GzEncoder;
use proc_macro2::TokenStream;
use quote::quote;
use rand::RngCore;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use syn::DeriveInput;
use walkdir::WalkDir;

pub fn expand_encrypted_embed(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident;

    let mut folder: Option<String> = None;
    let mut key_hex: Option<String> = None;
    let mut prefix: Option<String> = None;
    let mut hidden_key = false;
    let mut no_compress = false;

    // Parse attributes
    for attr in input.attrs {
        if let Ok(meta) = attr.parse_meta() {
            match meta {
                syn::Meta::NameValue(meta) => {
                    if meta.path.is_ident("folder") {
                        if let syn::Lit::Str(lit) = meta.lit {
                            folder = Some(lit.value());
                        }
                    } else if meta.path.is_ident("key") {
                        if let syn::Lit::Str(lit) = meta.lit {
                            key_hex = Some(lit.value());
                        }
                    } else if meta.path.is_ident("prefix") {
                        if let syn::Lit::Str(lit) = meta.lit {
                            prefix = Some(lit.value());
                        }
                    }
                }
                syn::Meta::Path(path) => {
                    if path.is_ident("hidden_key") {
                        hidden_key = true;
                    } else if path.is_ident("no_compress") {
                        no_compress = true;
                    }
                }
                _ => {}
            }
        }
    }

    if hidden_key && key_hex.is_none() {
        return Err(syn::Error::new_spanned(
            &ident,
            "Cannot use #[hidden_key] without an explicit #[key = \"...\"] attribute. A random key would be lost.",
        ));
    }

    let folder_path = folder
        .ok_or_else(|| syn::Error::new_spanned(&ident, "Missing #[folder = \"...\"] attribute"))?;
    // Resolve folder path relative to CARGO_MANIFEST_DIR
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new_spanned(&ident, "CARGO_MANIFEST_DIR not set"))?;
    let root = PathBuf::from(manifest_dir).join(&folder_path);

    if !root.exists() {
        return Err(syn::Error::new_spanned(
            &ident,
            format!("Folder does not exist: {:?}", root),
        ));
    }

    let mut rng = rand::thread_rng();

    // Determine key mode
    let key_option: Option<[u8; 32]> = match key_hex {
        Some(s) if s.is_empty() => None, // Explicitly no encryption
        Some(s) => {
            let mut key = [0u8; 32];
            hex::decode_to_slice(&s, &mut key)
                .map_err(|e| syn::Error::new_spanned(&ident, format!("Invalid hex key: {}", e)))?;
            Some(key)
        }
        None => {
            let mut key = [0u8; 32];
            rng.fill_bytes(&mut key);
            Some(key)
        }
    };

    let cipher = if let Some(key) = key_option {
        Some(XChaCha20Poly1305::new(GenericArray::from_slice(&key)))
    } else {
        None
    };

    let mut match_arms = Vec::new();
    let mut asset_keys = Vec::new();
    let mut dependency_paths = Vec::new();

    for entry in WalkDir::new(&root) {
        let entry = entry.map_err(|e| {
            syn::Error::new_spanned(&ident, format!("Error walking directory: {}", e))
        })?;
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        dependency_paths.push(path.to_string_lossy().into_owned());
        let relative_path = path.strip_prefix(&root).unwrap();
        let relative_path_str = relative_path.to_string_lossy().into_owned();

        // Handle prefix
        let asset_path = if let Some(p) = &prefix {
            format!("{}{}", p, relative_path_str)
        } else {
            relative_path_str
        };
        // Ensure forward slashes
        let asset_path = asset_path.replace("\\", "/");

        let content = fs::read(path).map_err(|e| {
            syn::Error::new_spanned(&ident, format!("Error reading file {:?}: {}", path, e))
        })?;

        // Compress (unless no_compress)
        let processed_content = if !no_compress {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&content).map_err(|e| {
                syn::Error::new_spanned(&ident, format!("Compression failed: {}", e))
            })?;
            encoder.finish().map_err(|e| {
                syn::Error::new_spanned(&ident, format!("Compression finish failed: {}", e))
            })?
        } else {
            content
        };

        let final_data = if let Some(cipher_instance) = &cipher {
            // Encrypt
            let mut nonce_bytes = [0u8; 24];
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = GenericArray::from_slice(&nonce_bytes);

            let ciphertext = cipher_instance
                .encrypt(nonce, processed_content.as_ref())
                .map_err(|e| {
                    syn::Error::new_spanned(&ident, format!("Encryption failed: {}", e))
                })?;

            // Combine nonce + ciphertext
            let mut encrypted_data = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
            encrypted_data.extend_from_slice(&nonce_bytes);
            encrypted_data.extend_from_slice(&ciphertext);
            encrypted_data
        } else {
            // No encryption
            processed_content
        };

        let bytes_lit = syn::LitByteStr::new(&final_data, proc_macro2::Span::call_site());

        match_arms.push(quote! {
            #asset_path => Some(std::borrow::Cow::Borrowed(#bytes_lit)),
        });
        asset_keys.push(asset_path);
    }

    // Generate include_bytes! calls to force Cargo to track file changes
    let dependencies = dependency_paths.iter().map(|path| {
        quote! {
            const _: &[u8] = include_bytes!(#path);
        }
    });

    let key_const = if !hidden_key {
        if let Some(k) = key_option {
            let k_lit = syn::LitByteStr::new(&k, proc_macro2::Span::call_site());
            quote! { Some(*#k_lit) }
        } else {
            quote! { None }
        }
    } else {
        quote! { None }
    };

    let is_encrypted = key_option.is_some();
    let is_compressed_const = !no_compress;

    Ok(quote! {
        #(#dependencies)*

        impl #ident {
            pub const KEY: Option<[u8; 32]> = #key_const;
            pub const IS_ENCRYPTED: bool = #is_encrypted;
            pub const IS_COMPRESSED: bool = #is_compressed_const;

            pub fn get(file_path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
                #[cfg(debug_assertions)]
                {
                    use std::fs;
                    use std::path::PathBuf;
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    // Assuming #folder_path is relative to manifest dir
                    let root = PathBuf::from(manifest_dir).join(#folder_path);

                    // Sanitize/normalize path logic could go here, but for simple concatenation:
                    // Note: file_path comes from the app, usually relative.
                    // We need to ensure it matches the structure expected.
                    let full_path = root.join(file_path);

                    if full_path.exists() && full_path.is_file() {
                         if let Ok(data) = fs::read(full_path) {
                             return Some(std::borrow::Cow::Owned(data));
                         }
                    }
                    None
                }

                #[cfg(not(debug_assertions))]
                match file_path {
                    #(#match_arms)*
                    _ => None,
                }
            }

            pub fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
                use std::borrow::Cow;
                const ITEMS: &[&str] = &[#(#asset_keys),*];
                ITEMS.iter().map(|s| Cow::Borrowed(*s))
            }
        }

        impl Embedable for #ident {
            fn get(file_path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
                #ident::get(file_path)
            }

            fn iter() -> alloc::boxed::Box<dyn Iterator<Item = std::borrow::Cow<'static, str>>> {
                alloc::boxed::Box::new(#ident::iter())
            }

            fn is_compressed() -> bool {
                #ident::IS_COMPRESSED
            }
        }
    })
}
