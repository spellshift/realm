mod kill_impl;
mod list_impl;

use std::sync::Arc;

use allocative::Allocative;
use anyhow::Result;
use derive_more::Display;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::dict::Dict;
use starlark::values::none::NoneType;
use starlark::values::{
    starlark_value, Heap, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};

use serde::{Serialize, Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "TasksLibrary")]
pub struct TasksLibrary();
starlark_simple_value!(TasksLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "tasks_library")]
impl<'v> StarlarkValue<'v> for TasksLibrary {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for TasksLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for TasksLibrary {
    fn expected() -> String {
        TasksLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<TasksLibrary>().unwrap())
    }
}

// This is where all of the "file.X" impl methods are bound
#[starlark_module]
#[rustfmt::skip]
fn methods(builder: &mut MethodsBuilder) {
    fn kill(this: TasksLibrary, task_id: i32) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        kill_impl::kill(task_id)?;
        Ok(NoneType{})
    }
    fn list<'v>(this: TasksLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> { //Should we use the JSON starlark type instead of String? Do I implement that here or somewhere else?
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        list_impl::list(starlark_heap)
    }
}
