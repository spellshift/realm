use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::{Context, Result};
use async_trait::async_trait;
use russh::client;
use russh::*;
use russh_keys::*;

pub struct Session {
    session: Arc<client::Handle<Client>>,
}

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        _server_public_key: &key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}

impl Session {
    pub async fn connect(
        user: String,
        password: Option<String>,
        key: Option<String>,
        key_password: Option<&str>,
        addrs: String,
    ) -> Result<Self> {
        let config = Arc::new(client::Config::default());
        let sh = Client {};
        let mut session = client::connect(config, addrs, sh).await?;
        let auth_res = if let Some(key_str) = key {
            let key_pair = if let Some(pw) = key_password {
                russh_keys::decode_secret_key(key_str.as_str(), Some(pw))?
            } else {
                russh_keys::decode_secret_key(key_str.as_str(), None)?
            };
            session.authenticate_publickey(user, Arc::new(key_pair)).await?
        } else if let Some(pw) = password {
            session.authenticate_password(user, pw).await?
        } else {
            return Err(anyhow::anyhow!("No password or key provided"));
        };

        if !auth_res {
            return Err(anyhow::anyhow!("Authentication failed"));
        }

        Ok(Self { session: Arc::new(session) })
    }

    pub async fn call(&self, command: &str) -> Result<russh::Channel<client::Msg>> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;
        Ok(channel)
    }

    pub async fn close(&self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "")
            .await?;
        Ok(())
    }

    pub async fn copy(&self, src: &str, dst: &str) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        let mut channel = self.session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await?;
        let sftp = russh_sftp::client::SftpSession::new(channel.into_stream()).await?;

        // Use sftp.create which returns a file handle
        // Wait, I should verify what create returns. In russh-sftp 2.0.8, it returns impl Future<Output = Result<File>>.
        // File has write_all.
        // It does not have close in some versions, but drop closes it.
        // The error `no method named close` confirms it doesn't have explicit close.
        let mut remote_file = sftp.create(dst).await?;
        let local_data = std::fs::read(src)?;
        remote_file.write_all(&local_data).await?;

        // Dropping remote_file closes it.
        Ok(())
    }
}
