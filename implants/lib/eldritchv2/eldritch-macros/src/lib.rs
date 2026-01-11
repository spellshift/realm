extern crate proc_macro;

use proc_macro::TokenStream;

mod encrypted_embed;
mod impls;
#[cfg(test)]
mod tests;

/// Derives an encrypted and compressed asset embedding implementation.
///
/// This macro provides a secure alternative to `rust_embed::RustEmbed` by compressing assets with Gzip
/// and optionally encrypting them with XChaCha20Poly1305 at compile-time.
///
/// # Attributes
///
/// * `#[folder = "path"]`: The directory containing assets to embed (relative to `CARGO_MANIFEST_DIR`). Required.
/// * `#[key = "hex"]`: A 32-byte (64 hex characters) encryption key.
///   - If omitted: A random key is generated at compile-time.
///   - If empty (`#[key = ""]`): Encryption is skipped (only compression is applied).
/// * `#[hidden_key]`: If present, the `KEY` constant will be set to `None`, ensuring the key bytes are not stored in the binary.
///   - Must be used with an explicit `#[key = "..."]`.
///   - Requires passing the key manually to `EncryptedEmbeddedAssets::new()` at runtime.
/// * `#[prefix = "prefix/"]`: An optional string to prepend to all asset paths (e.g. "dist/" or "assets/").
///
/// # Generated Constants
///
/// * `pub const KEY: Option<[u8; 32]>`: The key used for encryption (useful for random keys). If `#[hidden_key]` is used, this is `None`.
/// * `pub const IS_ENCRYPTED: bool`: Whether encryption was applied.
///
/// # Example
///
/// ```rust
/// #[derive(EncryptedEmbed)]
/// #[folder = "assets/"]
/// #[prefix = "static/"]
/// pub struct MyAssets;
/// ```
#[proc_macro_derive(EncryptedEmbed, attributes(folder, key, prefix))]
pub fn derive_encrypted_embed(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    match encrypted_embed::expand_encrypted_embed(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn eldritch_library(attr: TokenStream, item: TokenStream) -> TokenStream {
    match impls::expand_eldritch_library(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn eldritch_library_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match impls::expand_eldritch_library_impl(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn eldritch_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
