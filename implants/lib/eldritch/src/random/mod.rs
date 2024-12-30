mod bool_impl;
mod int_impl;
mod string_impl;

use starlark::{environment::MethodsBuilder, starlark_module, values::starlark_value};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(RandomLibrary, "random_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn bool<'v>(this: &RandomLibrary) -> anyhow::Result<bool> {
        bool_impl::bool()
    }

    #[allow(unused_variables)]
    fn int<'v>(this: &RandomLibrary, min: i32, max: i32) -> anyhow::Result<i32> {
        int_impl::int(min, max)
    }

    #[allow(unused_variables)]
    fn string<'v>(this: &RandomLibrary, length: u64, charset: Option<String>) -> anyhow::Result<String> {
        string_impl::string(length, charset)
    }
}
