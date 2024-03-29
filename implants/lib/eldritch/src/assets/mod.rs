mod copy_impl;
mod list_impl;
mod read_binary_impl;
mod read_impl;

use rust_embed::RustEmbed;
use starlark::{environment::MethodsBuilder, eval::Evaluator, values::none::NoneType};
use starlark_derive::{starlark_module, starlark_value};

#[cfg(debug_assertions)]
#[derive(RustEmbed)]
#[folder = "../../../bin/embedded_files_test"]
pub struct Asset;

#[cfg(not(feature = "imix"))]
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../../implants/golem/embed_files_golem_prod"]
pub struct Asset;

#[cfg(feature = "imix")]
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../../implants/imix/install_scripts"]
pub struct Asset;

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(AssetsLibrary, "assets_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn copy<'v>(this: &AssetsLibrary, starlark_eval: &mut Evaluator<'v, '_>, src: String, dest: String) -> anyhow::Result<NoneType> {
        copy_impl::copy(starlark_eval, src, dest)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn list(this: &AssetsLibrary, starlark_eval: &mut Evaluator<'v, '_>) -> anyhow::Result<Vec<String>> {
        list_impl::list(starlark_eval)
    }

    #[allow(unused_variables)]
    fn read_binary(this: &AssetsLibrary, src: String) -> anyhow::Result<Vec<u32>> {
        read_binary_impl::read_binary(src)
    }

    #[allow(unused_variables)]
    fn read(this: &AssetsLibrary, starlark_eval: &mut Evaluator<'v, '_>, src: String) -> anyhow::Result<String> {
        read_impl::read(starlark_eval, src)
    }
}
