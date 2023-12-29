mod copy_impl;
mod list_impl;
mod read_binary_impl;
mod read_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::{
    starlark_value, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
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

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "AssetsLibrary")]
pub struct AssetsLibrary();
starlark_simple_value!(AssetsLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "assets_library")]
impl<'v> StarlarkValue<'v> for AssetsLibrary {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for AssetsLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for AssetsLibrary {
    fn expected() -> String {
        AssetsLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<AssetsLibrary>().unwrap())
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
        list_impl::list()
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
