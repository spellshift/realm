mod exec_impl;
mod is_linux_impl;
mod is_windows_impl;
mod is_macos_impl;
mod shell_impl;

use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

#[derive(Copy, Clone, Debug, PartialEq, Display)]
#[display(fmt = "SysLibrary")]
pub struct SysLibrary();
starlark_simple_value!(SysLibrary);

impl<'v> StarlarkValue<'v> for SysLibrary {
    starlark_type!("sys_library");

    fn get_methods(&self) -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
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
    fn exec(_this: SysLibrary, path: String, args: Vec<String>, disown: Option<bool>) -> String {
        exec_impl::exec(path, args, disown)
    }
    fn is_linux(_this: SysLibrary) -> bool {
        is_linux_impl::is_linux()
    }
    fn is_windows(_this: SysLibrary) -> bool {
        is_windows_impl::is_windows()
    }
    fn is_macos(_this: SysLibrary) -> bool {
        is_macos_impl::is_macos()
    }
    fn shell(_this: SysLibrary, cmd: String) -> String {
        shell_impl::shell(cmd)
    }
}