extern crate proc_macro;

use proc_macro::TokenStream;

mod impls;
#[cfg(test)]
mod tests;

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
