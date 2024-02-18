mod format_to_epoch_impl;
mod format_to_readable_impl;
mod now_impl;
mod sleep_impl;

use starlark::{
    environment::MethodsBuilder,
    starlark_module,
    values::{none::NoneType, starlark_value},
};

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(TimeLibrary, "time_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn now<'v>(this: &TimeLibrary) -> anyhow::Result<u64> {
        now_impl::now()
    }

    #[allow(unused_variables)]
    fn sleep<'v>(this: &TimeLibrary, secs: f64) -> anyhow::Result<NoneType> {
        sleep_impl::sleep(secs);
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn format_to_epoch<'v>(this: &TimeLibrary, s: &str, fmt: &str) -> anyhow::Result<u64> {
        format_to_epoch_impl::format_to_epoch(s, fmt)
    }

    #[allow(unused_variables)]
    fn format_to_readable<'v>(this: &TimeLibrary, t: i64, fmt: &str) -> anyhow::Result<String> {
        format_to_readable_impl::format_to_readable(t, fmt)
    }
}
