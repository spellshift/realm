mod exec_impl;
mod is_linux_impl;
mod is_windows_impl;
mod is_macos_impl;
mod shell_impl;

use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType)]
#[display(fmt = "SysLibrary")]
pub struct SysLibrary();
starlark_simple_value!(SysLibrary);

impl<'v> StarlarkValue<'v> for SysLibrary {
    starlark_type!("sys_library");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for SysLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for SysLibrary {
    fn expected() -> String {
        SysLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<SysLibrary>().unwrap())
    }
}

// This is where all of the "sys.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn exec(this: SysLibrary, path: String, args: Vec<String>, disown: Option<bool>) -> anyhow::Result<String> {
        exec_impl::exec(path, args, disown)
    }
    fn is_linux(this: SysLibrary) -> anyhow::Result<bool> {
        is_linux_impl::is_linux()
    }
    fn is_windows(this: SysLibrary) -> anyhow::Result<bool> {
        is_windows_impl::is_windows()
    }
    fn is_macos(this: SysLibrary) -> anyhow::Result<bool> {
        is_macos_impl::is_macos()
    }
    fn shell(this: SysLibrary, cmd: String) -> anyhow::Result<String> {
        shell_impl::shell(cmd)
    }
}