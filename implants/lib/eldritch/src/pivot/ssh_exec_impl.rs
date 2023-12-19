use anyhow::Result;
use starlark::{
    collections::SmallMap,
    values::{dict::Dict, Heap},
};

use super::super::insert_dict_kv;
use super::Session;
use starlark::const_frozen_string;

struct SSHExecOutput {
    stdout: String,
    status: i32,
}

async fn handle_ssh_exec(
    target: String,
    port: u16,
    command: String,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<&str>,
    timeout: Option<u32>,
) -> Result<SSHExecOutput> {
    let mut ssh = tokio::time::timeout(
        std::time::Duration::from_secs(timeout.unwrap_or(3).try_into()?),
        Session::connect(
            username,
            password,
            key,
            key_password,
            format!("{}:{}", target, port),
        ),
    )
    .await??;
    let r = ssh.call(&command).await?;
    ssh.close().await?;

    Ok(SSHExecOutput {
        stdout: r.output()?,
        status: r.code.unwrap_or(0) as i32,
    })
}

pub fn ssh_exec(
    starlark_heap: &Heap,
    target: String,
    port: i32,
    command: String,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<String>,
    timeout: Option<u32>,
) -> Result<Dict> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let key_password_ref = key_password.as_deref();
    let local_port: u16 = port.try_into()?;

    let cmd_res = match runtime.block_on(handle_ssh_exec(
        target,
        local_port,
        command,
        username,
        password,
        key,
        key_password_ref,
        timeout,
    )) {
        Ok(local_res) => local_res,
        Err(local_err) => {
            return Err(anyhow::anyhow!(
                "Failed to run handle_ssh_exec: {}",
                local_err.to_string()
            ))
        }
    };

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    insert_dict_kv!(dict_res, starlark_heap, "stdout", &cmd_res.stdout, String);
    insert_dict_kv!(dict_res, starlark_heap, "status", cmd_res.status, i32);

    Ok(dict_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use russh::server::{Auth, Msg, Session};
    use russh::*;
    use russh_keys::*;
    use std::collections::HashMap;
    use std::process::Command;
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tokio::task;

    // SSH Server utils
    #[derive(Clone)]
    #[allow(dead_code)]
    struct Server {
        client_pubkey: Arc<russh_keys::key::PublicKey>,
        clients: Arc<Mutex<HashMap<(usize, ChannelId), Channel<Msg>>>>,
        id: usize,
    }

    impl server::Server for Server {
        type Handler = Self;
        fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
            let s = self.clone();
            self.id += 1;
            s
        }
    }

    #[async_trait]
    impl server::Handler for Server {
        type Error = anyhow::Error;

        async fn channel_open_session(
            self,
            channel: Channel<Msg>,
            session: Session,
        ) -> Result<(Self, bool, Session), Self::Error> {
            {
                let mut clients = self.clients.lock().unwrap();
                clients.insert((self.id, channel.id()), channel);
            }
            Ok((self, true, session))
        }

        #[allow(unused_variables)]
        async fn exec_request(
            self,
            channel: ChannelId,
            data: &[u8],
            mut session: Session,
        ) -> Result<(Self, Session), Self::Error> {
            let cmd = std::str::from_utf8(data)?;

            let command_string: &str;
            let command_args: Vec<&str>;

            if cfg!(target_os = "macos") {
                command_string = "bash";
                command_args = ["-c", cmd].to_vec();
            } else if cfg!(target_os = "windows") {
                command_string = "cmd";
                command_args = ["/c", cmd].to_vec();
            } else if cfg!(target_os = "linux") {
                command_string = "bash";
                command_args = ["-c", cmd].to_vec();
            } else {
                // linux and such
                command_string = "bash";
                command_args = ["-c", cmd].to_vec();
            }
            let tmp_res = Command::new(command_string).args(command_args).output()?;
            session.data(channel, CryptoVec::from(tmp_res.stdout));
            session.close(channel); // Only gonna send one command.
            Ok((self, session))
        }

        #[allow(unused_variables)]
        async fn auth_publickey(
            self,
            _: &str,
            _: &key::PublicKey,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, server::Auth::Accept))
        }

        #[allow(unused_variables)]
        async fn auth_password(
            self,
            user: &str,
            password: &str,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, Auth::Accept))
        }

        async fn data(
            self,
            _channel: ChannelId,
            data: &[u8],
            mut session: Session,
        ) -> Result<(Self, Session), Self::Error> {
            {
                let mut clients = self.clients.lock().unwrap();
                for ((_, _channel_id), ref mut channel) in clients.iter_mut() {
                    session.data(channel.id(), CryptoVec::from(data.to_vec()));
                }
            }
            Ok((self, session))
        }
    }

    async fn test_ssh_server(address: String, port: u16) {
        let client_key = russh_keys::key::KeyPair::generate_ed25519().unwrap();
        let client_pubkey = Arc::new(client_key.clone_public_key().unwrap());
        let mut config = russh::server::Config::default();
        config.connection_timeout = Some(std::time::Duration::from_secs(3));
        config.auth_rejection_time = std::time::Duration::from_secs(3);
        config
            .keys
            .push(russh_keys::key::KeyPair::generate_ed25519().unwrap());
        let config = Arc::new(config);
        let sh = Server {
            client_pubkey,
            clients: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
        };
        let _res = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            russh::server::run(config, (address, port), sh),
        )
        .await
        .unwrap_or(Ok(()));
    }

    // Tests run concurrently so each test needs a unique port.
    async fn allocate_localhost_unused_ports() -> anyhow::Result<i32> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        Ok(listener.local_addr().unwrap().port().into())
    }

    #[tokio::test]
    async fn test_pivot_ssh_exec() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let ssh_command = r#"echo "hello world""#.to_string();
        let test_server_task = task::spawn(test_ssh_server(ssh_host.clone(), ssh_port));

        let key_pass = "test123";
        let ssh_client_task = task::spawn(
            handle_ssh_exec(ssh_host.clone(), ssh_port.into(), ssh_command, "root".to_string(), Some("some_password".to_string()), Some(String::from("-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABAXll5Hd2\nu/V1Bl4vNt07NNAAAAEAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIPfYgoW3Oh7quQgG\nzuRLHeEzMVyex2D8l0dwPPKmAF9EAAAAoOtSZeeMu8IOVfJyA6aEqrbvmRoCIwT5EHOEzu\nzDu1n3j/ud0bZZORxa0UhREbde0cvg5SEpwmLu1iiR3apRN0CHhE7+fv790IGnQ/y1Dc0M\n1zHU6/luG5Nc83fZPtREiPqaOwPlyxI1xXALk9dvn4m+jv4cMdxZqrKsNX7sIeTZoI3PIt\nrwIiywheU2wKsnw3WDMCTXAKkB0FYOv4tosBY=\n-----END OPENSSH PRIVATE KEY-----")), Some(key_pass), Some(2))
        );

        let (_a, actual_response) = tokio::join!(test_server_task, ssh_client_task);
        let res = actual_response??;
        assert!(res.stdout.contains("hello world"));
        Ok(())
    }
}
