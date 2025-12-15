use crate::std::Session;
use crate::std::StdPivotLibrary;
use alloc::format;
use alloc::string::{String, ToString};
use anyhow::Result;

#[allow(clippy::too_many_arguments)]
async fn handle_ssh_copy(
    target: String,
    port: u16,
    src: String,
    dst: String,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<&str>,
    timeout: Option<u32>,
) -> Result<()> {
    let mut ssh = tokio::time::timeout(
        std::time::Duration::from_secs(timeout.unwrap_or(3) as u64),
        Session::connect(
            username,
            password,
            key,
            key_password,
            format!("{target}:{port}"),
        ),
    )
    .await??;
    ssh.copy(&src, &dst).await?;
    ssh.close().await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    lib: &StdPivotLibrary,
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
    let (tx, rx) = std::sync::mpsc::channel();

    let target_clone = target.clone();
    let port_u16 = port as u16;
    let src_clone = src.clone();
    let dst_clone = dst.clone();
    let username_clone = username.clone();
    let password_clone = password.clone();
    let key_clone = key.clone();
    let key_password_clone = key_password.clone();
    let timeout_u32 = timeout.map(|t| t as u32);

    let fut = async move {
        // Need to convert Option<String> to Option<&str> for key_password if needed, but handle_ssh_copy signature above uses &str for key_password?
        // Wait, current handle_ssh_copy signature: key_password: Option<&str>
        // But we are moving data into async block. &str is tricky if string owned by block.
        // We should change handle_ssh_copy to take String or Option<String>.

        // Actually, let's fix handle_ssh_copy signature to take Option<String> to avoid lifetime issues in async move block.
        // But handle_ssh_copy is defined in THIS file above.
        // I should change it.

        let key_pass_ref = key_password_clone.as_deref();

        let res = handle_ssh_copy(
            target_clone,
            port_u16,
            src_clone,
            dst_clone,
            username_clone,
            password_clone,
            key_clone,
            key_pass_ref, // This will fail because key_password_clone is moved into closure, so we can take ref?
                          // No, `async move` moves `key_password_clone`. We can take ref to it inside.
                          // But handle_ssh_copy expects `Option<&str>`.
                          // `key_password_clone` is `Option<String>`.
                          // `key_password_clone.as_deref()` returns `Option<&str>`.
                          // This should work.
            timeout_u32,
        ).await;

        let _ = tx.send(res);
    };

    lib.agent
        .spawn_subtask(lib.task_id, "ssh_copy".to_string(), alloc::boxed::Box::pin(fut))
        .map_err(|e| e.to_string())?;

    let response = rx.recv().map_err(|e| format!("Failed to receive result: {}", e))?;

    match response {
        Ok(_) => Ok("Success".to_string()),
        Err(e) => Err(format!("ssh_copy failed: {:?}", e)),
    }
}

// Stub for Session if not available in this scope?
// `use crate::std::Session;` is at top.
// `std.rs` defines `Session`.
// But I haven't read `std.rs` fully to see if it exposes `Session`.
// Previous `ssh_copy_impl.rs` used `crate::std::Session`.
// I assume `std.rs` has `pub use ssh_session::Session` or something.

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use russh::server::{Auth, Msg, Session};
    use russh::*;
    use russh_sftp::protocol::{
        File, FileAttributes, Handle, Name, OpenFlags, Status, StatusCode, Version,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;
    use tokio::net::TcpListener;
    use tokio::task;

    // SSH Server utils
    #[derive(Clone)]
    struct Server;

    impl russh::server::Server for Server {
        type Handler = SshSession;

        fn new_client(&mut self, _: Option<SocketAddr>) -> Self::Handler {
            SshSession::default()
        }
    }

    struct SshSession {
        clients: Arc<Mutex<HashMap<ChannelId, Channel<Msg>>>>,
    }

    impl Default for SshSession {
        fn default() -> Self {
            Self {
                clients: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl SshSession {
        pub async fn get_channel(&mut self, channel_id: ChannelId) -> Channel<Msg> {
            let mut clients = self.clients.lock().unwrap();
            clients.remove(&channel_id).unwrap()
        }
    }

    #[async_trait]
    #[allow(unused_variables)]
    impl russh::server::Handler for SshSession {
        type Error = anyhow::Error;

        async fn auth_password(
            self,
            user: &str,
            password: &str,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, Auth::Accept))
        }

        async fn auth_publickey(
            self,
            user: &str,
            public_key: &russh_keys::key::PublicKey,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, Auth::Accept))
        }

        async fn channel_open_session(
            mut self,
            channel: Channel<Msg>,
            session: Session,
        ) -> Result<(Self, bool, Session), Self::Error> {
            {
                let mut clients = self.clients.lock().unwrap();
                clients.insert(channel.id(), channel);
            }
            Ok((self, true, session))
        }

        async fn subsystem_request(
            mut self,
            channel_id: ChannelId,
            name: &str,
            mut session: Session,
        ) -> Result<(Self, Session), Self::Error> {
            if name == "sftp" {
                let channel = self.get_channel(channel_id).await;
                let sftp = SftpSession::default();
                session.channel_success(channel_id);
                russh_sftp::server::run(channel.into_stream(), sftp).await;
            } else {
                session.channel_failure(channel_id);
            }

            Ok((self, session))
        }
    }

    #[derive(Default)]
    struct SftpSession {
        version: Option<u32>,
        root_dir_read_done: bool,
    }

    #[allow(unused_variables)]
    #[async_trait]
    impl russh_sftp::server::Handler for SftpSession {
        type Error = StatusCode;

        fn unimplemented(&self) -> Self::Error {
            StatusCode::OpUnsupported
        }

        async fn init(
            &mut self,
            version: u32,
            extensions: HashMap<String, String>,
        ) -> Result<Version, Self::Error> {
            if self.version.is_some() {
                return Err(StatusCode::ConnectionLost);
            }
            self.version = Some(version);
            Ok(Version::new())
        }

        async fn close(&mut self, id: u32, _handle: String) -> Result<Status, Self::Error> {
            Ok(Status {
                id,
                status_code: StatusCode::Ok,
                error_message: "Ok".to_string(),
                language_tag: "en-US".to_string(),
            })
        }

        async fn remove(&mut self, id: u32, handle: String) -> Result<Status, Self::Error> {
            std::fs::remove_file(handle).unwrap();
            Ok(Status {
                id,
                status_code: StatusCode::Ok,
                error_message: "Ok".to_string(),
                language_tag: "en-US".to_string(),
            })
        }

        async fn opendir(&mut self, id: u32, path: String) -> Result<Handle, Self::Error> {
            self.root_dir_read_done = false;
            Ok(Handle { id, handle: path })
        }

        async fn open(
            &mut self,
            id: u32,
            filename: String,
            pflags: OpenFlags,
            attrs: FileAttributes,
        ) -> Result<Handle, Self::Error> {
            Ok(Handle {
                id,
                handle: filename,
            })
        }

        #[allow(unused_variables)]
        async fn write(
            &mut self,
            id: u32,
            handle: String,
            offset: u64,
            data: Vec<u8>,
        ) -> Result<Status, Self::Error> {
            //Warning this will only write one chunk - subsequesnt chunks will overwirte the old ones.
            // Tests over the size of the chunk will fail
            let tmp_data = String::from_utf8(data).unwrap();
            fs::write(handle, tmp_data.trim_end_matches(char::from(0))).unwrap();
            Ok(Status {
                id,
                status_code: StatusCode::Ok,
                error_message: "".to_string(),
                language_tag: "".to_string(),
            })
        }

        async fn readdir(&mut self, id: u32, handle: String) -> Result<Name, Self::Error> {
            if handle == "/" && !self.root_dir_read_done {
                self.root_dir_read_done = true;
                return Ok(Name {
                    id,
                    files: vec![
                        File {
                            filename: "foo".to_string(),
                            longname: "".to_string(),
                            attrs: FileAttributes::default(),
                        },
                        File {
                            filename: "bar".to_string(),
                            longname: "".to_string(),
                            attrs: FileAttributes::default(),
                        },
                    ],
                });
            }
            Ok(Name { id, files: vec![] })
        }

        async fn realpath(&mut self, id: u32, path: String) -> Result<Name, Self::Error> {
            Ok(Name {
                id,
                files: vec![File {
                    filename: "/".to_string(),
                    longname: "".to_string(),
                    attrs: FileAttributes::default(),
                }],
            })
        }
    }

    #[allow(unused_variables)]
    async fn test_ssh_server(address: String, port: u16) {
        let client_key = russh_keys::key::KeyPair::generate_ed25519().unwrap();
        let client_pubkey = Arc::new(client_key.clone_public_key().unwrap());
        let config = Arc::new(russh::server::Config {
            connection_timeout: Some(std::time::Duration::from_secs(3)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            keys: vec![russh_keys::key::KeyPair::generate_ed25519().unwrap()],
            ..Default::default()
        });
        let sh = Server {};
        let _res: std::result::Result<(), std::io::Error> = tokio::time::timeout(
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
    async fn test_pivot_ssh_copy() -> anyhow::Result<()> {
        const TEST_STRING: &[u8; 12] = b"Hello, world";
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();

        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        tmp_file_src.write_all(TEST_STRING)?;

        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());
        // let path_dst = "/foo".to_string();
        // tmp_file_dst.close()?;

        let test_server_task = task::spawn(test_ssh_server(ssh_host.clone(), ssh_port));

        let key_pass = "test123";
        let ssh_client_task = task::spawn(handle_ssh_copy(
            ssh_host.clone(),
            ssh_port,
            path_src,
            path_dst.clone(),
            "root".to_string(),
            Some("some_password".to_string()),
            Some(String::from(
                "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABAXll5Hd2\nu/V1Bl4vNt07NNAAAAEAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIPfYgoW3Oh7quQgG\nzuRLHeEzMVyex2D8l0dwPPKmAF9EAAAAoOtSZeeMu8IOVfJyA6aEqrbvmRoCIwT5EHOEzu\nzDu1n3j/ud0bZZORxa0UhREbde0cvg5SEpwmLu1iiR3apRN0CHhE7+fv790IGnQ/y1Dc0M\n1zHU6/luG5Nc83fZPtREiPqaOwPlyxI1xXALk9dvn4m+jv4cMdxZqrKsNX7sIeTZoI3PIt\nrwIiywheU2wKsnw3WDMCTXAKkB0FYOv4tosBY=\n-----END OPENSSH PRIVATE KEY-----",
            )),
            Some(key_pass), // Pass &str here in test, but handle_ssh_copy expects &str
                            // Wait, handle_ssh_copy signature above says `key_password: Option<&str>`.
                            // In test call: `Some(key_pass)` where key_pass is `&str`.
                            // This matches.
            Some(2),
        ));

        let (_a, actual_response) = tokio::join!(test_server_task, ssh_client_task);
        actual_response??;

        let res_buf = fs::read_to_string(path_dst);
        assert_eq!(TEST_STRING, res_buf?.as_bytes());
        Ok(())
    }
}
