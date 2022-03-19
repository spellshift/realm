mod kill_impl;
mod list_impl;
mod name_impl;

use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
use starlark::values::none::NoneType;
use starlark::{starlark_type, starlark_simple_value, starlark_module};

#[derive(Copy, Clone, Debug, PartialEq, Display)]
#[display(fmt = "ProcessLibrary")]
pub struct ProcessLibrary();
starlark_simple_value!(ProcessLibrary);

impl<'v> StarlarkValue<'v> for ProcessLibrary {
    starlark_type!("process_library");

    fn get_methods(&self) -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
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
    fn kill(_this: ProcessLibrary, pid: i32) -> NoneType {
        kill_impl::kill(pid)?;
        Ok(NoneType{})
    }
    fn list(_this: ProcessLibrary) -> Vec<String> {
        list_impl::list()
    }
    fn name(_this: ProcessLibrary, pid: i32) -> String {
        name_impl::name(pid)
    }
}