use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use anyhow::Result;
use crate::PivotLibrary;
use crate::arp_scan_impl;
use crate::ncat_impl;
use crate::port_scan_impl;
use crate::reverse_shell_pty_impl;
use crate::ssh_copy_impl;
use crate::ssh_exec_impl;

// SSH Client utils
use async_trait::async_trait;
use russh::{client, Disconnect};
use russh_keys::{decode_secret_key, key};
use russh_sftp::client::SftpSession;
use std::sync::Arc;
use alloc::string::ToString;
use eldritch_macros::eldritch_library_impl;

// Deps for Agent
use eldritch_libagent::agent::Agent;
use transport::SyncTransport;

#[derive(Default)]
#[eldritch_library_impl(PivotLibrary)]
pub struct StdPivotLibrary {
    pub agent: Option<Arc<dyn Agent>>,
    pub transport: Option<Arc<dyn SyncTransport>>,
    pub task_id: Option<i64>,
}

impl core::fmt::Debug for StdPivotLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdPivotLibrary")
         .field("task_id", &self.task_id)
         .finish()
    }
}

impl StdPivotLibrary {
    pub fn new(agent: Arc<dyn Agent>, transport: Arc<dyn SyncTransport>, task_id: i64) -> Self {
        Self {
            agent: Some(agent),
            transport: Some(transport),
            task_id: Some(task_id),
        }
    }
}

impl PivotLibrary for StdPivotLibrary {
    fn reverse_shell_pty(&self, cmd: Option<String>) -> Result<(), String> {
        let transport = self.transport.as_ref().ok_or_else(|| "No transport available".to_string())?;
        let task_id = self.task_id.ok_or_else(|| "No task_id available".to_string())?;
        reverse_shell_pty_impl::reverse_shell_pty(transport.clone(), task_id, cmd).map_err(|e| e.to_string())
    }

    fn reverse_shell_repl(&self) -> Result<(), String> {
        // Not implemented fully yet as per instructions, or should use transport too.
        // User didn't specify repl logic changes but implied pivot handles it.
        // Assuming similar to PTY for now but might need separate impl.
        Err("REPL reverse shell not fully migrated".to_string())
    }

    fn ssh_exec(
        &self,
        target: String,
        port: i64,
        command: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<BTreeMap<String, Value>, String> {
        ssh_exec_impl::ssh_exec(target, port as i32, command, username, password, key, key_password, timeout.map(|t| t as u32)).map_err(|e| e.to_string())
    }

    fn ssh_copy(
        &self,
        target: String,
        port: i64,
        src: String,
        dst: String,
        username: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<String>,
        timeout: Option<i64>,
    ) -> Result<String, String> {
        ssh_copy_impl::ssh_copy(target, port as i32, src, dst, username, password, key, key_password, timeout.map(|t| t as u32)).map_err(|e| e.to_string())
    }

    fn port_scan(
        &self,
        target_cidrs: Vec<String>,
        ports: Vec<i64>,
        protocol: String,
        timeout: i64,
        fd_limit: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let ports_i32: Vec<i32> = ports.into_iter().map(|p| p as i32).collect();
        port_scan_impl::port_scan(target_cidrs, ports_i32, protocol, timeout as i32, fd_limit).map_err(|e| e.to_string())
    }

    fn arp_scan(&self, target_cidrs: Vec<String>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        arp_scan_impl::arp_scan(target_cidrs).map_err(|e| e.to_string())
    }

    fn ncat(
        &self,
        address: String,
        port: i64,
        data: String,
        protocol: String,
    ) -> Result<String, String> {
        ncat_impl::ncat(address, port as i32, data, protocol).map_err(|e| e.to_string())
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
    pub async fn connect(
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

    pub async fn copy(&mut self, src: &str, dst: &str) -> anyhow::Result<()> {
        let mut channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await.unwrap();
        let sftp = SftpSession::new(channel.into_stream()).await.unwrap();

        let _ = sftp.remove_file(dst).await;
        let mut dst_file = sftp.create(dst).await?;
        let mut src_file = tokio::io::BufReader::new(tokio::fs::File::open(src).await?);
        let _bytes_copied = tokio::io::copy_buf(&mut src_file, &mut dst_file).await?;

        Ok(())
    }

    pub async fn call(&mut self, command: &str) -> anyhow::Result<CommandResult> {
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

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

pub struct CommandResult {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<u32>,
}

impl CommandResult {
    pub fn output(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.stdout).to_string())
    }

    pub fn error(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.stderr).to_string())
    }
}
