mod kill_impl;
mod list_impl;
mod info_impl;
mod name_impl;
mod netstat_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::dict::Dict;
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType, Heap, starlark_value};
use starlark::{starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "ProcessLibrary")]
pub struct ProcessLibrary();
starlark_simple_value!(ProcessLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "process_library")]
impl<'v> StarlarkValue<'v> for ProcessLibrary {

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
    fn list<'v>(this: ProcessLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> { //Should we use the JSON starlark type instead of String? Do I implement that here or somewhere else?
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        list_impl::list(starlark_heap)
    }
    fn info<'v>(this: ProcessLibrary, starlark_heap: &'v Heap, pid: Option<usize>) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        info_impl::info(starlark_heap, pid)
    }
    fn name(this: ProcessLibrary, pid: i32) -> anyhow::Result<String> {
         if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
         name_impl::name(pid)
    }
    fn netstat<'v>(this: ProcessLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        netstat_impl::netstat(starlark_heap)
    }
}
