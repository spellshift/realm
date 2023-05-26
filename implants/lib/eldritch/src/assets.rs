mod copy_impl;
mod list_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};
use rust_embed::RustEmbed;

#[cfg(debug_assertions)]
#[derive(RustEmbed)]
#[folder = "../../bin/embedded_files_test"]
pub struct Asset;

#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../../implants/golem/embed_files_golem_prod"]
pub struct Asset;


#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "AssetsLibrary")]
pub struct AssetsLibrary();
starlark_simple_value!(AssetsLibrary);

impl<'v> StarlarkValue<'v> for AssetsLibrary {
    starlark_type!("assets_library");

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
}