mod dll_inject_impl;
mod dll_reflect_impl;
mod exec_impl;
mod get_env_impl;
mod get_ip_impl;
mod get_os_impl;
mod get_pid_impl;
mod get_reg_impl;
mod get_user_impl;
mod hostname_impl;
mod is_linux_impl;
mod is_macos_impl;
mod is_windows_impl;
mod shell_impl;
mod write_reg_hex_impl;
mod write_reg_int_impl;
mod write_reg_str_impl;

use allocative::Allocative;
use derive_more::Display;

use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::starlark_value;
use starlark::values::{
    dict::Dict, Heap, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};

use serde::{Serialize, Serializer};

struct CommandOutput {
    stdout: String,
    stderr: String,
    status: i32,
}

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
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    fn exec<'v>(this: SysLibrary, starlark_heap: &'v Heap, path: String, args: Vec<String>, disown: Option<bool>) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        exec_impl::exec(starlark_heap, path, args, disown)
    }
    fn get_os<'v>(this: SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_os_impl::get_os(starlark_heap)
    }
    fn dll_inject(this: SysLibrary, dll_path: String, pid: u32) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        dll_inject_impl::dll_inject(dll_path, pid)
    }
    fn dll_reflect(this: SysLibrary, dll_bytes: Vec<u32>, pid: u32, function_name: String) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        dll_reflect_impl::dll_reflect(dll_bytes, pid, function_name)
    }
    fn get_env<'v>(this: SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_env_impl::get_env(starlark_heap)
    }
    fn get_ip<'v>(this: SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_ip_impl::get_ip(starlark_heap)
    }
    fn get_pid<'v>(this: SysLibrary) -> anyhow::Result<u32> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_pid_impl::get_pid()
    }
    fn get_user<'v>(this: SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_user_impl::get_user(starlark_heap)
    }
    fn hostname(this: SysLibrary) -> anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        hostname_impl::hostname()
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
    fn shell<'v>(this:  SysLibrary, starlark_heap: &'v Heap, cmd: String) ->  anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        shell_impl::shell(starlark_heap, cmd)
    }
    fn get_reg<'v>(this:  SysLibrary, starlark_heap: &'v Heap, reghiv: String, regpth: String) ->  anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        get_reg_impl::get_reg(starlark_heap, reghiv, regpth)
    }
    fn write_reg_str(this:  SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: String) ->  anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        write_reg_str_impl::write_reg_str(reghiv, regpth, regname, regtype, regvalue)
    }
    fn write_reg_int(this:  SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: u32) ->  anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        write_reg_int_impl::write_reg_int(reghiv, regpth, regname, regtype, regvalue)
    }
    fn write_reg_hex(this:  SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: String) ->  anyhow::Result<bool> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        write_reg_hex_impl::write_reg_hex(reghiv, regpth, regname, regtype, regvalue)
    }
}
