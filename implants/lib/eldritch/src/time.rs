mod sleep_impl;
mod now_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::starlark_value;
use starlark::values::{ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike};
use starlark::{starlark_module, starlark_simple_value};

use serde::{Serialize, Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "TimeLibrary")]
pub struct TimeLibrary();
starlark_simple_value!(TimeLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "sys_library")]
impl<'v> StarlarkValue<'v> for TimeLibrary {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for TimeLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for TimeLibrary {
    fn expected() -> String {
        TimeLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<TimeLibrary>().unwrap())
    }
}

// This is where all of the "Time.X" impl methods are bound
#[starlark_module]
#[rustfmt::skip]
fn methods(builder: &mut MethodsBuilder) {
    fn now<'v>(this: TimeLibrary) -> anyhow::Result<u64> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        now_impl::now()
    }
    fn sleep<'v>(this: TimeLibrary, secs: f64) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        sleep_impl::sleep(secs);
        Ok(NoneType{})
    }
}
