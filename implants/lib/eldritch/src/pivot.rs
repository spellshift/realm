mod ssh_exec_impl;
mod ssh_password_spray_impl;
mod smb_exec_impl;
mod port_scan_impl;
mod arp_scan_impl;
mod port_forward_impl;
mod ncat_impl;
mod bind_proxy_impl;

use std::io::Write;
use std::sync::Arc;

use allocative::Allocative;
use async_trait::async_trait;
use derive_more::Display;

use russh::{client, Disconnect};
use russh_keys::{key, decode_secret_key};
use starlark::values::dict::Dict;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::values::none::NoneType;
use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike, ProvidesStaticType, Heap};
use starlark::{starlark_type, starlark_simple_value, starlark_module};

use serde::{Serialize,Serializer};
use tokio::net::ToSocketAddrs;

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
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
    fn ssh_exec<'v>(this: PivotLibrary, starlark_heap: &'v Heap, target: String, port: i32, command: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<u32>) ->  anyhow::Result<Dict<'v>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        ssh_exec_impl::ssh_exec(starlark_heap, target, port, command, username, password, key, key_password, timeout)
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
    fn port_scan<'v>(this:  PivotLibrary, starlark_heap: &'v Heap, target_cidrs: Vec<String>, ports: Vec<i32>, protocol: String, timeout:  i32) ->  anyhow::Result<Vec<Dict<'v>>> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        port_scan_impl::port_scan(starlark_heap, target_cidrs, ports, protocol, timeout)
    }
    fn arp_scan<'v>(
        this: PivotLibrary,
        starlark_heap: &'v Heap,
        target_cidrs: Vec<String>,
    ) -> anyhow::Result<Vec<Dict<'v>>> {
        if false {
            println!(
                "Ignore unused this var. _this isn't allowed by starlark. {:?}",
                this
            );
        }
        arp_scan_impl::arp_scan(starlark_heap, target_cidrs)
    }
    fn port_forward(this:  PivotLibrary, listen_address: String, listen_port: i32, forward_address: String, forward_port: i32, protocol: String) ->  anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        port_forward_impl::port_forward(listen_address, listen_port, forward_address, forward_port, protocol)?;
        Ok(NoneType{})
    }
    fn ncat(this:  PivotLibrary, address: String, port: i32, data: String, protocol: String) ->  anyhow::Result<String> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        ncat_impl::ncat(address, port, data, protocol)
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



// SSH Client utils
struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}

pub struct Session {
    session: client::Handle<Client>,
}

impl Session {
    async fn connect(
        user: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<&str>,
        addrs: String,
    ) -> anyhow::Result<Self> {
        let config = client::Config {
            ..<_>::default()
        };
        let config = Arc::new(config);
        let sh = Client {};
        let mut session = client::connect(config, addrs.clone(), sh).await?;

        // Try key auth first
        match key {
            Some(local_key) => {
                let key_pair = decode_secret_key(&local_key, key_password)?;
                let _auth_res: bool = session
                    .authenticate_publickey(user, Arc::new(key_pair))
                    .await?;
                return Ok(Self { session });
            },
            None => {},
        }

        // If key auth doesn't work try password auth
        match password {
            Some(local_pass) => {
                let _auth_res: bool = session
                    .authenticate_password(user, local_pass)
                    .await?;
                return Ok(Self { session });
            },
            None => {},
        }
        return Err(anyhow::anyhow!("Failed to authenticate to host {}@{}", user, addrs.clone()));
    }

    async fn call(&mut self, command: &str) -> anyhow::Result<CommandResult> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut output = Vec::new();
        let mut code = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { ref data } => {
                    output.write_all(data).unwrap();
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                _ => {
                }
            }
        }
        Ok(CommandResult { output, code })
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

struct CommandResult {
    output: Vec<u8>,
    code: Option<u32>,
}

impl CommandResult {
    fn output(&self) -> String {
        String::from_utf8_lossy(&self.output).into()
    }
}

