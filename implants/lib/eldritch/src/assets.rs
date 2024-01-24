mod copy_impl;
mod list_impl;
mod read_binary_impl;
mod read_impl;

use allocative::Allocative;
use derive_more::Display;

use c2::pb::c2_manual_client::TavernClient;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::{
    starlark_value, NoSerialize, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};

use rust_embed::RustEmbed;
use serde::{Serialize, Serializer};

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

#[derive(Debug, Display, Clone, ProvidesStaticType, NoSerialize, Allocative)]
#[display(fmt = "AssetsLibrary")]
pub(crate) struct AssetsLibrary(#[allocative(skip)] pub TavernClient);
starlark_simple_value!(AssetsLibrary);

// #[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
// #[display(fmt = "AssetsLibrary")]
// pub struct AssetsLibrary {
//     #[allocative(skip)]
//     pub embedded_assets: String,
// }
// starlark_simple_value!(AssetsLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "assets_library")]
impl<'v> StarlarkValue<'v> for AssetsLibrary {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for AssetsLibrary {
    fn expected() -> String {
        AssetsLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        match AssetsLibrary::from_value(value) {
            Some(x) => Some(x.clone()),
            None => None,
        }
    }
}

// This is where all of the "assets.X" impl methods are bound
#[starlark_module]
#[rustfmt::skip]
fn methods(builder: &mut MethodsBuilder) {
    fn copy(this: AssetsLibrary, src: String, dest: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        copy_impl::copy(src, dest)?;
        Ok(NoneType{})
    }
    fn list(this: AssetsLibrary) -> anyhow::Result<Vec<String>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        list_impl::list(this)
    }
    fn read_binary(this: AssetsLibrary, src: String) -> anyhow::Result<Vec<u32>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        read_binary_impl::read_binary(src)
    }
    fn read(this: AssetsLibrary, src: String) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        read_impl::read(src)
    }

}
