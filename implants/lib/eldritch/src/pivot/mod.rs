mod arp_scan_impl;
mod bind_proxy_impl;
mod ncat_impl;
mod port_forward_impl;
mod port_scan_impl;
mod reverse_shell_pty_impl;
mod smb_exec_impl;
mod ssh_copy_impl;
mod ssh_exec_impl;

use anyhow::Result;
use async_trait::async_trait;
use russh::{client, Disconnect};
use russh_keys::{decode_secret_key, key};
use russh_sftp::client::SftpSession;
use starlark::{
    environment::MethodsBuilder,
    eval::Evaluator,
    starlark_module,
    values::{dict::Dict, list::UnpackList, none::NoneType, starlark_value, Heap},
};
use std::sync::Arc;

/*
 * Define our library for this module.
 */
crate::eldritch_lib!(PivotLibrary, "pivot_library");

/*
 * Below, we define starlark wrappers for all of our library methods.
 * The functions must be defined here to be present on our library.
 */
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    #[allow(unused_variables)]
    fn reverse_shell_pty(this: &PivotLibrary, starlark_eval: &mut Evaluator<'v, '_>, cmd: Option<String>) -> anyhow::Result<NoneType> {
        let env = crate::runtime::Environment::from_extra(starlark_eval.extra)?;
        reverse_shell_pty_impl::reverse_shell_pty(env, cmd)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn ssh_exec<'v>(this: &PivotLibrary, starlark_heap: &'v Heap, target: String, port: i32, command: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<u32>) ->  anyhow::Result<Dict<'v>> {
        ssh_exec_impl::ssh_exec(starlark_heap, target, port, command, username, password, key, key_password, timeout)
    }

    #[allow(unused_variables)]
    fn ssh_copy<'v>(this: &PivotLibrary, target: String, port: i32, src: String, dst: String, username: String, password: Option<String>, key: Option<String>, key_password: Option<String>, timeout: Option<u32>) ->  anyhow::Result<String> {
        ssh_copy_impl::ssh_copy(target, port, src, dst, username, password, key, key_password, timeout)
    }

    #[allow(unused_variables)]
    fn smb_exec(this: &PivotLibrary, target: String, port: i32, username: String, password: String, hash: String, command: String) ->  anyhow::Result<String> {
        smb_exec_impl::smb_exec(target, port, username, password, hash, command)
    }

    #[allow(unused_variables)]
    fn port_scan<'v>(this: &PivotLibrary, starlark_heap: &'v Heap, target_cidrs: UnpackList<String>, ports: UnpackList<i32>, protocol: String, timeout:  i32) ->  anyhow::Result<Vec<Dict<'v>>> {
        // May want these too: PSRemoting, WMI, WinRM
        port_scan_impl::port_scan(starlark_heap, target_cidrs.items, ports.items, protocol, timeout)
    }

    #[allow(unused_variables)]
    fn arp_scan<'v>(
        this: &PivotLibrary,
        starlark_heap: &'v Heap,
        target_cidrs: UnpackList<String>,
    ) -> anyhow::Result<Vec<Dict<'v>>> {
        arp_scan_impl::arp_scan(starlark_heap, target_cidrs.items)
    }

    #[allow(unused_variables)]
    fn port_forward(this: &PivotLibrary, listen_address: String, listen_port: i32, forward_address: String, forward_port: i32, protocol: String) ->  anyhow::Result<NoneType> {
        port_forward_impl::port_forward(listen_address, listen_port, forward_address, forward_port, protocol)?;
        Ok(NoneType{})
    }

    #[allow(unused_variables)]
    fn ncat(this: &PivotLibrary, address: String, port: i32, data: String, protocol: String, timeout: Option<u32>) ->  anyhow::Result<String> {
        ncat_impl::ncat(address, port, data, protocol, timeout)
    }

    #[allow(unused_variables)]
    fn bind_proxy(this: &PivotLibrary, listen_address: String, listen_port: i32, username: String, password: String) ->  anyhow::Result<NoneType> {
        // Seems to have the best protocol support - https://github.com/ajmwagar/merino
        bind_proxy_impl::bind_proxy(listen_address, listen_port, username, password)?;
        Ok(NoneType{})
    }
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
        let config = client::Config { ..<_>::default() };
        let config = Arc::new(config);
        let sh = Client {};
        let mut session = client::connect(config, addrs.clone(), sh).await?;

        // Try key auth first
        if let Some(local_key) = key {
            let key_pair = decode_secret_key(&local_key, key_password)?;
            let _auth_res: bool = session
                .authenticate_publickey(user, Arc::new(key_pair))
                .await?;
            return Ok(Self { session });
        }

        // If key auth doesn't work try password auth
        if let Some(local_pass) = password {
            let _auth_res: bool = session.authenticate_password(user, local_pass).await?;
            return Ok(Self { session });
        }

        Err(anyhow::anyhow!(
            "Failed to authenticate to host {}@{}",
            user,
            addrs.clone()
        ))
    }

    async fn copy(&mut self, src: &str, dst: &str) -> anyhow::Result<()> {
        let mut channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await.unwrap();
        let sftp = SftpSession::new(channel.into_stream()).await.unwrap();

        let _ = sftp.remove_file(dst).await;
        let mut dst_file = sftp.create(dst).await?;
        let mut src_file = tokio::io::BufReader::new(tokio::fs::File::open(src).await?);
        let _bytes_copied = tokio::io::copy_buf(&mut src_file, &mut dst_file).await?;

        Ok(())
    }

    async fn call(&mut self, command: &str) -> anyhow::Result<CommandResult> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut code = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { ref data } => {
                    std::io::Write::write_all(&mut stdout, data).unwrap();
                }
                russh::ChannelMsg::ExtendedData { ref data, ext: _ } => {
                    std::io::Write::write_all(&mut stderr, data).unwrap();
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                _ => {}
            }
        }
        Ok(CommandResult {
            stdout,
            stderr,
            code,
        })
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

struct CommandResult {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    code: Option<u32>,
}

impl CommandResult {
    fn output(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.stdout).to_string())
    }

    fn error(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.stderr).to_string())
    }
}
