mod ssh_exec_impl;
mod ssh_password_spray_impl;
mod smb_exec_impl;
mod port_scan_impl;
mod arp_scan_impl;
mod port_forward_impl;
mod ncat_impl;
mod bind_proxy_impl;

use derive_more::Display;

use starlark::values::dict::Dict;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::list::List;
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType)]
#[display(fmt = "PivotLibrary")]
pub struct PivotLibrary();
starlark_simple_value!(PivotLibrary);

impl<'v> StarlarkValue<'v> for PivotLibrary {
    starlark_type!("pivot_library");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for PivotLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for PivotLibrary {
    fn expected() -> String {
        PivotLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<PivotLibrary>().unwrap())
    }
}

// This is where all of the "file.X" impl methods are bound
#[starlark_module]
fn methods(builder: &mut MethodsBuilder) {
    fn ssh_exec(this:  PivotLibrary, target: String, port: i32, username: String, password: String, key: String, command: String, shell_path: String) ->  anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }        
        ssh_exec_impl::ssh_exec(target, port, username, password, key, command, shell_path)
    }
    fn ssh_password_spray(this:  PivotLibrary, targets: Vec<String>, port: i32, credentials: Vec<String>, keys: Vec<String>, command: String, shell_path: String) ->  anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        ssh_password_spray_impl::ssh_password_spray(targets, port, credentials, keys, command, shell_path)
    }
    fn smb_exec(this:  PivotLibrary, target: String, port: i32, username: String, password: String, hash: String, command: String) ->  anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        smb_exec_impl::smb_exec(target, port, username, password, hash, command)
    }     
    // May want these too: PSRemoting, WMI, WinRM
    fn port_scan(this:  PivotLibrary, target_cidrs: Vec<String>, ports: Vec<i32>, portocol: String, timeout:  i32) ->  anyhow::Result<Vec<Dict>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        port_scan_impl::port_scan(target_cidrs, ports, portocol, timeout)
    }
    fn arp_scan(this:  PivotLibrary, target_cidrs: Vec<String>) ->  anyhow::Result<Vec<String>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        arp_scan_impl::arp_scan(target_cidrs)
    }
    fn port_forward(this:  PivotLibrary, listen_address: String, listen_port: i32, forward_address: String, forward_port: i32, portocol: String) ->  anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        port_forward_impl::port_forward(listen_address, listen_port, forward_address, forward_port, portocol)?;
        Ok(NoneType{})
    }
    fn ncat(this:  PivotLibrary, address: String, port: i32, data: String, portocol: String) ->  anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        ncat_impl::ncat(address, port, data, portocol)
    }
    // Seems to have the best protocol support - https://github.com/ajmwagar/merino
    fn bind_proxy(this:  PivotLibrary, listen_address: String, listen_port: i32, username: String, password: String) ->  anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        bind_proxy_impl::bind_proxy(listen_address, listen_port, username, password)?;
        Ok(NoneType{})
    }

    // This + smb_copy should likely move to file or rolled into the download function  or made into an upload function.
    // fn ssh_copy(_this:  PivotLibrary, target: String, port: i32, username: String, password: String, key: String, src: String, dst: String) ->  String {
    //   ssh_copy_impl::ssh_copy(target, port, username, password, key, command, shell_path, src, dst)?;
    //   Ok(NoneType{})
    // }
}