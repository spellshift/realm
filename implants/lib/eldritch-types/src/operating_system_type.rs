use allocative_derive::Allocative;
use derive_more::Display;
use starlark::environment::{MethodsBuilder, MethodsStatic, Methods};
use starlark::values::{UnpackValue, Value, ValueLike};
use starlark::{starlark_simple_value, values::StarlarkValue};
use starlark_derive::NoSerialize;
use starlark_derive::starlark_module;
use starlark_derive::starlark_value;
use starlark_derive::ProvidesStaticType;

#[derive(Clone, Debug, PartialEq, Eq, Display, ProvidesStaticType, NoSerialize, Allocative)]
#[display(fmt = "(arch: {}, desktop_env: {}, distro: {}, platform: {})", arch, desktop_env, distro, platform)]
pub struct OperatingSystemType {
    pub arch: String,
    pub desktop_env: String,
    pub distro: String,
    pub platform: String,
}
starlark_simple_value!(OperatingSystemType);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "operating_system_type")]
impl<'v> StarlarkValue<'v> for OperatingSystemType {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl<'v> UnpackValue<'v> for OperatingSystemType {
    fn expected() -> String {
        OperatingSystemType::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        let tmp = value.downcast_ref::<OperatingSystemType>().unwrap();
        Some(OperatingSystemType { 
            arch: tmp.arch.clone(), 
            desktop_env: tmp.desktop_env.clone(), 
            distro: tmp.distro.clone(),
            platform: tmp.platform.clone(),
        })
    }
}


#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn arch(this: OperatingSystemType) -> anyhow::Result<String> {
        Ok(this.arch)
    }
    fn desktop_env(this: OperatingSystemType) -> anyhow::Result<String> {
        Ok(this.desktop_env)
    }
    fn distro(this: OperatingSystemType) -> anyhow::Result<String> {
        Ok(this.distro)
    }
    fn platform(this: OperatingSystemType) -> anyhow::Result<String> {
        Ok(this.platform)
    }
}