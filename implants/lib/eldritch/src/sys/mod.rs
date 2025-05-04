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
mod is_bsd_impl;
mod is_linux_impl;
mod is_macos_impl;
mod is_windows_impl;
mod shell_impl;
mod write_reg_hex_impl;
mod write_reg_int_impl;
mod write_reg_str_impl;

use starlark::{
    collections::SmallMap,
    environment::MethodsBuilder,
    starlark_module,
    values::{dict::Dict, list::UnpackList, none::NoneType, starlark_value, Heap},
};

struct CommandOutput {
    stdout: String,
    stderr: String,
    status: i32,
}

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(SysLibrary, "sys_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn exec<'v>(this: &SysLibrary, starlark_heap: &'v Heap, path: String, args: UnpackList<String>, disown: Option<bool>, env_vars: Option<SmallMap<String, String>>) -> anyhow::Result<Dict<'v>> {
        exec_impl::exec(starlark_heap, path, args.items, disown, env_vars)
    }

    #[allow(unused_variables)]
    fn get_os<'v>(this: &SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        get_os_impl::get_os(starlark_heap)
    }

    #[allow(unused_variables)]
    fn dll_inject(this: &SysLibrary, dll_path: String, pid: u32) -> anyhow::Result<NoneType> {
        dll_inject_impl::dll_inject(dll_path, pid)
    }

    #[allow(unused_variables)]
    fn dll_reflect(this: &SysLibrary, dll_bytes: UnpackList<u32>, pid: u32, function_name: String) -> anyhow::Result<NoneType> {
        dll_reflect_impl::dll_reflect(dll_bytes.items, pid, function_name)
    }

    #[allow(unused_variables)]
    fn get_env<'v>(this: &SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        get_env_impl::get_env(starlark_heap)
    }

    #[allow(unused_variables)]
    fn get_ip<'v>(this: &SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Vec<Dict<'v>>> {
        get_ip_impl::get_ip(starlark_heap)
    }

    #[allow(unused_variables)]
    fn get_pid<'v>(this: &SysLibrary) -> anyhow::Result<u32> {
        get_pid_impl::get_pid()
    }

    #[allow(unused_variables)]
    fn get_user<'v>(this: &SysLibrary, starlark_heap: &'v Heap) -> anyhow::Result<Dict<'v>> {
        get_user_impl::get_user(starlark_heap)
    }

    #[allow(unused_variables)]
    fn hostname(this: &SysLibrary) -> anyhow::Result<String> {
        hostname_impl::hostname()
    }

    #[allow(unused_variables)]
    fn is_bsd(this: &SysLibrary) -> anyhow::Result<bool> {
        is_bsd_impl::is_bsd()
    }

    #[allow(unused_variables)]
    fn is_linux(this: &SysLibrary) -> anyhow::Result<bool> {
        is_linux_impl::is_linux()
    }

    #[allow(unused_variables)]
    fn is_windows(this: &SysLibrary) -> anyhow::Result<bool> {
        is_windows_impl::is_windows()
    }

    #[allow(unused_variables)]
    fn is_macos(this: &SysLibrary) -> anyhow::Result<bool> {
        is_macos_impl::is_macos()
    }

    #[allow(unused_variables)]
    fn shell<'v>(this: &SysLibrary, starlark_heap: &'v Heap, cmd: String) ->  anyhow::Result<Dict<'v>> {
        shell_impl::shell(starlark_heap, cmd)
    }

    #[allow(unused_variables)]
    fn get_reg<'v>(this: &SysLibrary, starlark_heap: &'v Heap, reghiv: String, regpth: String) ->  anyhow::Result<Dict<'v>> {
        get_reg_impl::get_reg(starlark_heap, reghiv, regpth)
    }

    #[allow(unused_variables)]
    fn write_reg_str(this: &SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: String) ->  anyhow::Result<bool> {
        write_reg_str_impl::write_reg_str(reghiv, regpth, regname, regtype, regvalue)
    }

    #[allow(unused_variables)]
    fn write_reg_int(this: &SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: u32) ->  anyhow::Result<bool> {
        write_reg_int_impl::write_reg_int(reghiv, regpth, regname, regtype, regvalue)
    }

    #[allow(unused_variables)]
    fn write_reg_hex(this: &SysLibrary, reghiv: String, regpth: String, regname: String, regtype: String, regvalue: String) ->  anyhow::Result<bool> {
        write_reg_hex_impl::write_reg_hex(reghiv, regpth, regname, regtype, regvalue)
    }
}
