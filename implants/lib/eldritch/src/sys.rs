mod exec_impl;
mod get_ip_impl;
mod get_os_impl;
mod is_linux_impl;
mod is_windows_impl;
mod is_macos_impl;
mod shell_impl;
mod dll_inject_impl;

use allocative::Allocative;
use derive_more::Display;

use eldritch_types::command_output::CommandOutput;
use eldritch_types::network_interface::NetworkInterface;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::starlark_value;
use starlark::values::{StarlarkValue, Value, Heap, dict::Dict, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "SysLibrary")]
pub struct SysLibrary();
starlark_simple_value!(SysLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "sys_library")]
impl<'v> StarlarkValue<'v> for SysLibrary {

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
    fn exec(this: SysLibrary, path: String, args: Vec<String>, disown: Option<bool>) -> anyhow::Result<CommandOutput> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        exec_impl::exec(path, args, disown)
    }
    fn get_os<'v>(this: SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_os_impl::get_os(starlark_heap)
    }
    fn dll_inject(this: SysLibrary, dll_path: String, pid: u32) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        dll_inject_impl::dll_inject(dll_path, pid)
    }
    fn get_ip<'v>(this: SysLibrary) -> anyhow::Result<Vec<NetworkInterface>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_ip_impl::get_ip()
    }
    fn is_linux(this: SysLibrary) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        is_linux_impl::is_linux()
    }
    fn is_windows(this: SysLibrary) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        is_windows_impl::is_windows()
    }
    fn is_macos(this: SysLibrary) -> anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        is_macos_impl::is_macos()
    }
    fn shell(this: SysLibrary, cmd: String) ->  anyhow::Result<CommandOutput> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        shell_impl::shell(cmd)
    }
}