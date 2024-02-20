mod download_impl;
mod get_impl;

use starlark::{
    collections::SmallMap,
    environment::MethodsBuilder,
    starlark_module,
    values::{none::NoneType, starlark_value, Value},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(HTTPLibrary, "http_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn download(this: &HTTPLibrary, uri: String, dst: String) -> anyhow::Result<NoneType> {
        download_impl::download(uri, dst)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn get(this: &HTTPLibrary, uri: String, query_params: Option<SmallMap<String, Value>>, headers: Option<SmallMap<String, String>>) -> anyhow::Result<String> {
        get_impl::get(uri, query_params, headers)
    }
}
