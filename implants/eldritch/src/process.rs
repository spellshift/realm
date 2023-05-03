mod kill_impl;
mod list_impl;
mod name_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "ProcessLibrary")]
pub struct ProcessLibrary();
starlark_simple_value!(ProcessLibrary);

impl<'v> StarlarkValue<'v> for ProcessLibrary {
    starlark_type!("process_library");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for ProcessLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for ProcessLibrary {
    fn expected() -> String {
        ProcessLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<ProcessLibrary>().unwrap())
    }
}

// This is where all of the "process.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn kill(this: ProcessLibrary, pid: i32) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        kill_impl::kill(pid)?;
        Ok(NoneType{})
    }
    fn list(this: ProcessLibrary) -> anyhow::Result<Vec<String>> { //Should we use the JSON starlark type instead of String? Do I implement that here or somewhere else?
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        list_impl::list()
    }
    fn name(this: ProcessLibrary, pid: i32) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        name_impl::name(pid)
    }
}