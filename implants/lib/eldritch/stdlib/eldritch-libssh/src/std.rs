use crate::SshLibrary;
pub mod copy_impl;
pub mod deploy_impl;
pub mod exec_impl;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result;
use async_trait::async_trait;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use russh::{Disconnect, client};
use russh_keys::{decode_secret_key, key};
use russh_sftp::client::SftpSession;
use std::sync::Arc;

#[derive(Default, Debug)]
#[eldritch_library_impl(SshLibrary)]
pub struct StdSshLibrary;

impl SshLibrary for StdSshLibrary {
    fn copy(
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
        copy_impl::ssh_copy(
            target,
            port as i32,
            src,
            dst,
            username,
            password,
            key,
            key_password,
            timeout.map(|t| t as u32),
        )
        .map_err(|e| e.to_string())
    }

    fn deploy(
        &self,
        ips: Vec<String>,
        credentials: Vec<BTreeMap<String, Value>>,
        cmd: String,
        privesc_cmd: Option<String>,
        payload: Option<Vec<u8>>,
        payload_dst: Option<String>,
        timeout: Option<i64>,
        retries: Option<i64>,
    ) -> Result<Vec<BTreeMap<String, Value>>, String> {
        deploy_impl::ssh_deploy(
            ips,
            credentials,
            cmd,
            privesc_cmd,
            payload,
            payload_dst,
            timeout,
            retries,
        )
        .map_err(|e| e.to_string())
    }

    fn exec(
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
        exec_impl::ssh_exec(
            target,
            port as i32,
            command,
            username,
            password,
            key,
            key_password,
            timeout.map(|t| t as u32),
        )
        .map_err(|e| e.to_string())
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
            let auth_res: bool = session
                .authenticate_publickey(user.clone(), Arc::new(key_pair))
                .await?;
            if !auth_res {
                return Err(anyhow::anyhow!(
                    "publickey authentication rejected for {}@{}",
                    user,
                    addrs
                ));
            }
            return Ok(Self { session });
        }

        // If key auth doesn't work try password auth
        if let Some(local_pass) = password {
            let auth_res: bool = session
                .authenticate_password(user.clone(), local_pass)
                .await?;
            if !auth_res {
                return Err(anyhow::anyhow!(
                    "password authentication rejected for {}@{}",
                    user,
                    addrs
                ));
            }
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

    pub async fn copy_bytes(&mut self, src: &[u8], dst: &str) -> anyhow::Result<()> {
        let mut channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await.unwrap();
        let sftp = SftpSession::new(channel.into_stream()).await.unwrap();

        let _ = sftp.remove_file(dst).await;
        let mut dst_file = sftp.create(dst).await?;
        let mut src_reader = src;
        let _bytes_copied = tokio::io::copy(&mut src_reader, &mut dst_file).await?;

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
