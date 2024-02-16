mod match_all_impl;
mod match_impl;
mod replace_all_impl;
mod replace_impl;

use starlark::{
    environment::MethodsBuilder,
    starlark_module,
    values::{none::NoneType, starlark_value},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(RegexLibrary, "regex_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn replace_all<'v>(this: &RegexLibrary, haystack: String, pattern: String, text: String) -> anyhow::Result<NoneType> {
        replace_all_impl::replace_all(haystack, pattern, text)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn replace<'v>(this: &RegexLibrary, haystack: String, pattern: String, text: String) -> anyhow::Result<NoneType> {
        replace_impl::replace(haystack, pattern, text)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn match_all<'v>(this: &RegexLibrary, haystack: String, pattern: String, text: String) -> anyhow::Result<Vec<String>> {
        match_all_impl::match_all(haystack, pattern, text)
    }

    #[allow(unused_variables)]
    fn r#match<'v>(this: &RegexLibrary, haystack: String, pattern: String, text: String) -> anyhow::Result<String> {
        match_impl::r#match(haystack, pattern, text)
    }
}
