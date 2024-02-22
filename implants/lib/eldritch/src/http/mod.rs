mod download_impl;
mod get_impl;
mod post_impl;

use starlark::{
    collections::SmallMap,
    environment::MethodsBuilder,
    starlark_module,
    values::{none::NoneType, starlark_value},
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
    fn download(this: &HTTPLibrary, uri: String, dst: String, allow_insecure: Option<bool>) -> anyhow::Result<NoneType> {
        download_impl::download(uri, dst, allow_insecure)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn get(this: &HTTPLibrary, uri: String, query_params: Option<SmallMap<String, String>>, headers: Option<SmallMap<String, String>>, allow_insecure: Option<bool>) -> anyhow::Result<String> {
        get_impl::get(uri, query_params, headers, allow_insecure)
    }

    #[allow(unused_variables)]
    fn post(this: &HTTPLibrary, uri: String, body: Option<String>, form: Option<SmallMap<String, String>>, headers: Option<SmallMap<String, String>>, allow_insecure: Option<bool>) -> anyhow::Result<String> {
        post_impl::post(uri, body, form, headers, allow_insecure)
    }
}
