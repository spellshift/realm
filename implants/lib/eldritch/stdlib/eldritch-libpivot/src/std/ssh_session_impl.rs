use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use anyhow::Result;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};

use crate::std::Session;

use std::sync::Mutex;

/// Persistent SSH session handle – <ssh_session> foreign value.
///
/// `inner` holds `Option<Session>` so `close()` can take it and future
/// `exec()` fails with "session closed". The `Mutex` is only held briefly;
/// we never hold it across an await – we take, await, then restore.
#[eldritch_library("ssh_session")]
pub trait SshSessionLibrary {
    #[eldritch_method]
    /// Executes a command on the remote host via the existing SSH transport.
    fn exec(&self, command: String) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Closes the SSH session, disconnecting the transport. Idempotent.
    fn close(&self) -> Result<(), String>;
}

#[eldritch_library_impl(SshSessionLibrary)]
pub struct SshSessionHandle {
    inner: Arc<Mutex<Option<Session>>>,
    target: String,
    port: i32,
}

impl core::fmt::Debug for SshSessionHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let connected = self.inner.lock().map(|g| g.is_some()).unwrap_or(false);
        f.debug_struct("SshSessionHandle")
            .field("target", &self.target)
            .field("port", &self.port)
            .field("connected", &connected)
            .finish()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Connect helper + factory
// ──────────────────────────────────────────────────────────────────────────────

async fn handle_connect(
    target: String,
    port: u16,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<&str>,
    timeout: Option<u32>,
) -> Result<Session> {
    let secs = timeout.unwrap_or(3) as u64;
    let addr = format!("{target}:{port}");
    let session = tokio::time::timeout(
        std::time::Duration::from_secs(secs),
        Session::connect(username, password, key, key_password, addr),
    )
    .await
    .map_err(|_| anyhow::anyhow!("SSH connection timed out after {secs}s"))??;
    Ok(session)
}

/// Factory called by `StdPivotLibrary::ssh_session`.
pub fn ssh_session(
    target: String,
    port: i32,
    username: String,
    password: Option<String>,
    key: Option<String>,
    key_password: Option<String>,
    timeout: Option<u32>,
) -> Result<Value> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let local_port: u16 = port
        .try_into()
        .map_err(|e| anyhow::anyhow!("invalid port {port}: {e}"))?;

    let sess = runtime.block_on(handle_connect(
        target.clone(),
        local_port,
        username,
        password,
        key,
        key_password.as_deref(),
        timeout,
    ))?;

    let handle = SshSessionHandle {
        inner: Arc::new(Mutex::new(Some(sess))),
        target,
        port,
    };

    Ok(Value::Foreign(Arc::new(handle)))
}

// ──────────────────────────────────────────────────────────────────────────────
// SshSessionLibrary impl (sync, callable from non-tokio thread – matches other pivot impls)
// ──────────────────────────────────────────────────────────────────────────────

impl SshSessionLibrary for SshSessionHandle {
    fn exec(&self, command: String) -> Result<BTreeMap<String, Value>, String> {
        // Take session out of mutex so we don't hold the lock across await.
        let mut sess = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "ssh_session mutex poisoned".to_string())?;
            guard
                .take()
                .ok_or_else(|| "ssh_session is closed".to_string())?
        };

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        let (res, sess) = runtime.block_on(async {
            let r = sess.call(&command).await;
            (r, sess)
        });

        // Always restore, even on error – keep session for close().
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(sess);
        }

        let cr = res.map_err(|e| e.to_string())?;

        let mut map = BTreeMap::new();
        map.insert(
            "stdout".into(),
            Value::String(cr.output().map_err(|e| e.to_string())?),
        );
        map.insert(
            "stderr".into(),
            Value::String(cr.error().map_err(|e| e.to_string())?),
        );
        map.insert("status".into(), Value::Int(cr.code.unwrap_or(0) as i64));
        Ok(map)
    }

    fn close(&self) -> Result<(), String> {
        let maybe_sess = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "ssh_session mutex poisoned".to_string())?;
            guard.take()
        };

        if let Some(mut sess) = maybe_sess {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            runtime.block_on(sess.close()).map_err(|e| e.to_string())?;
        }
        // idempotent: already None -> success
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Async helpers for tests
// ──────────────────────────────────────────────────────────────────────────────

impl SshSessionHandle {
    #[cfg(test)]
    pub(crate) fn wrap_for_test(session: Session, target: String, port: i32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(session))),
            target,
            port,
        }
    }

    #[cfg(test)]
    pub(crate) async fn exec_async(
        &self,
        command: &str,
    ) -> anyhow::Result<BTreeMap<String, Value>> {
        let mut sess = {
            let mut guard = self.inner.lock().map_err(|_| anyhow::anyhow!("poisoned"))?;
            guard
                .take()
                .ok_or_else(|| anyhow::anyhow!("ssh_session is closed"))?
        };

        let res = sess.call(command).await;

        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(sess);
        }

        let cr = res?;
        let mut map = BTreeMap::new();
        map.insert("stdout".into(), Value::String(cr.output()?));
        map.insert("stderr".into(), Value::String(cr.error()?));
        map.insert("status".into(), Value::Int(cr.code.unwrap_or(0) as i64));
        Ok(map)
    }

    #[cfg(test)]
    pub(crate) async fn close_async(&self) -> anyhow::Result<()> {
        let maybe_sess = {
            let mut g = self.inner.lock().map_err(|_| anyhow::anyhow!("poisoned"))?;
            g.take()
        };
        if let Some(mut sess) = maybe_sess {
            sess.close().await?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn is_closed(&self) -> bool {
        self.inner.lock().map(|g| g.is_none()).unwrap_or(true)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use russh::server::{Auth, Msg, Session as ServerSession};
    use russh::*;
    use std::collections::HashMap;
    use std::process::Command;
    use std::sync::Mutex as StdMutex;
    use std::time::Duration;
    use tokio::net::TcpListener;
    use tokio::task;

    #[derive(Clone)]
    #[allow(dead_code)]
    struct TestServer {
        client_pubkey: Arc<russh_keys::key::PublicKey>,
        clients: Arc<StdMutex<HashMap<(usize, ChannelId), Channel<Msg>>>>,
        id: usize,
    }

    impl server::Server for TestServer {
        type Handler = Self;
        fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
            let s = self.clone();
            self.id += 1;
            s
        }
    }

    #[async_trait]
    impl server::Handler for TestServer {
        type Error = anyhow::Error;

        async fn channel_open_session(
            self,
            channel: Channel<Msg>,
            session: ServerSession,
        ) -> Result<(Self, bool, ServerSession), Self::Error> {
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
            mut session: ServerSession,
        ) -> Result<(Self, ServerSession), Self::Error> {
            let cmd = std::str::from_utf8(data)?;
            let (command_string, command_args): (&str, Vec<&str>) = if cfg!(target_os = "windows") {
                ("cmd", vec!["/c", cmd])
            } else {
                ("bash", vec!["-c", cmd])
            };
            let tmp_res = Command::new(command_string).args(command_args).output()?;
            session.data(channel, CryptoVec::from(tmp_res.stdout.clone()));
            if !tmp_res.stderr.is_empty() {
                session.extended_data(channel, 1, CryptoVec::from(tmp_res.stderr));
            }
            session.eof(channel);
            session.close(channel);
            Ok((self, session))
        }

        async fn auth_publickey(
            self,
            _: &str,
            _: &russh_keys::key::PublicKey,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, server::Auth::Accept))
        }

        async fn auth_password(
            self,
            _user: &str,
            _password: &str,
        ) -> Result<(Self, Auth), Self::Error> {
            Ok((self, Auth::Accept))
        }

        async fn data(
            self,
            _channel: ChannelId,
            data: &[u8],
            mut session: ServerSession,
        ) -> Result<(Self, ServerSession), Self::Error> {
            {
                let mut clients = self.clients.lock().unwrap();
                for ((_, _channel_id), ref mut channel) in clients.iter_mut() {
                    session.data(channel.id(), CryptoVec::from(data.to_vec()));
                }
            }
            Ok((self, session))
        }
    }

    async fn test_ssh_server_multi(address: String, port: u16, secs: u64) {
        let client_key = russh_keys::key::KeyPair::generate_ed25519().unwrap();
        let client_pubkey = Arc::new(client_key.clone_public_key().unwrap());
        let config = Arc::new(russh::server::Config {
            connection_timeout: Some(Duration::from_secs(10)),
            auth_rejection_time: Duration::from_secs(1),
            keys: vec![russh_keys::key::KeyPair::generate_ed25519().unwrap()],
            ..Default::default()
        });
        let sh = TestServer {
            client_pubkey,
            clients: Arc::new(StdMutex::new(HashMap::new())),
            id: 0,
        };
        let _ = tokio::time::timeout(
            Duration::from_secs(secs),
            russh::server::run(config, (address, port), sh),
        )
        .await;
    }

    async fn allocate_localhost_unused_ports() -> anyhow::Result<i32> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        Ok(listener.local_addr().unwrap().port().into())
    }

    /// Wait until TCP port is accepting connections, or time out.
    async fn wait_until_listening(host: &str, port: u16) {
        for _ in 0..50 {
            if tokio::net::TcpStream::connect(format!("{host}:{port}"))
                .await
                .is_ok()
            {
                // Small grace period: server accepted, but SSH stack may still be initializing.
                tokio::time::sleep(Duration::from_millis(50)).await;
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn test_ssh_session_connect_and_double_call_direct() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 8));
        wait_until_listening(&ssh_host, ssh_port).await;

        let mut sess = tokio::time::timeout(
            Duration::from_secs(3),
            Session::connect(
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                format!("{ssh_host}:{ssh_port}"),
            ),
        )
        .await??;

        let r1 = sess.call("echo first").await?;
        assert!(r1.output()?.contains("first"), "r1 was {:?}", r1.output());
        let r2 = sess.call("echo second").await?;
        assert!(r2.output()?.contains("second"), "r2 was {:?}", r2.output());

        sess.close().await?;
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_ssh_session_handle_reuse_async() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 8));
        wait_until_listening(&ssh_host, ssh_port).await;

        let sess = tokio::time::timeout(
            Duration::from_secs(3),
            Session::connect(
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                format!("{ssh_host}:{ssh_port}"),
            ),
        )
        .await??;

        let handle = SshSessionHandle::wrap_for_test(sess, ssh_host.clone(), ssh_port as i32);

        let r1 = handle.exec_async("echo alpha").await?;
        let out1 = match r1.get("stdout") {
            Some(Value::String(s)) => s.clone(),
            other => panic!("unexpected stdout: {other:?}"),
        };
        assert!(out1.contains("alpha"), "out1 was {out1:?}");

        let r2 = handle.exec_async("echo beta").await?;
        let out2 = match r2.get("stdout") {
            Some(Value::String(s)) => s.clone(),
            other => panic!("unexpected stdout: {other:?}"),
        };
        assert!(out2.contains("beta"), "out2 was {out2:?}");

        handle.close_async().await?;
        assert!(handle.is_closed());
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_ssh_session_handle_close_then_exec_fails() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 8));
        wait_until_listening(&ssh_host, ssh_port).await;

        let sess = tokio::time::timeout(
            Duration::from_secs(3),
            Session::connect(
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                format!("{ssh_host}:{ssh_port}"),
            ),
        )
        .await??;

        let handle = SshSessionHandle::wrap_for_test(sess, ssh_host.clone(), ssh_port as i32);
        handle.close_async().await?;

        let err = handle.exec_async("echo should-fail").await.unwrap_err();
        assert!(
            err.to_string().contains("closed"),
            "expected closed error, got {err:?}"
        );
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_ssh_session_factory_sync_success() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 10));
        wait_until_listening(&ssh_host, ssh_port).await;

        let port_for_blocking = ssh_port;
        let ssh_host_clone = ssh_host.clone();
        let val = task::spawn_blocking(move || {
            ssh_session(
                ssh_host_clone,
                port_for_blocking as i32,
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                Some(3),
            )
        })
        .await??;

        match &val {
            Value::Foreign(f) => {
                assert_eq!(f.type_name(), "ssh_session");
                let mut names = f.method_names();
                names.sort();
                assert!(names.contains(&"exec".to_string()), "methods: {names:?}");
                assert!(names.contains(&"close".to_string()), "methods: {names:?}");
            }
            other => panic!("expected Foreign, got {other:?}"),
        }

        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_ssh_session_factory_fail() -> anyhow::Result<()> {
        let free_port = allocate_localhost_unused_ports().await? as u16;
        let res = task::spawn_blocking(move || {
            ssh_session(
                "127.0.0.1".to_string(),
                free_port as i32,
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                Some(1),
            )
        })
        .await?;
        assert!(
            res.is_err(),
            "expected error when connecting to unused port"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_debug_no_secret_leak() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 5));
        wait_until_listening(&ssh_host, ssh_port).await;

        let sess = tokio::time::timeout(
            Duration::from_secs(3),
            Session::connect(
                "root".to_string(),
                Some("supersecretpassword".to_string()),
                None,
                None,
                format!("{ssh_host}:{ssh_port}"),
            ),
        )
        .await??;

        let handle = SshSessionHandle::wrap_for_test(sess, ssh_host.clone(), ssh_port as i32);
        let dbg = format!("{handle:?}");
        assert!(
            !dbg.contains("supersecretpassword"),
            "Debug leaked password: {dbg}"
        );
        assert!(
            dbg.contains("127.0.0.1"),
            "Debug should contain target: {dbg}"
        );

        handle.close_async().await?;
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_ssh_session_factory_multi_exec_then_close() -> anyhow::Result<()> {
        let ssh_port = allocate_localhost_unused_ports().await? as u16;
        let ssh_host = "127.0.0.1".to_string();
        let server_task = task::spawn(test_ssh_server_multi(ssh_host.clone(), ssh_port, 10));
        wait_until_listening(&ssh_host, ssh_port).await;

        let sess = tokio::time::timeout(
            Duration::from_secs(3),
            Session::connect(
                "root".to_string(),
                Some("pass".to_string()),
                None,
                None,
                format!("{ssh_host}:{ssh_port}"),
            ),
        )
        .await??;

        let handle = SshSessionHandle::wrap_for_test(sess, ssh_host.clone(), ssh_port as i32);

        let r1 = handle.exec_async("echo one").await?;
        let r2 = handle.exec_async("echo two").await?;
        let r3 = handle.exec_async("echo three").await?;

        let get_out = |m: &BTreeMap<String, Value>| match m.get("stdout") {
            Some(Value::String(s)) => s.clone(),
            other => panic!("bad stdout {other:?}"),
        };

        assert!(get_out(&r1).contains("one"));
        assert!(get_out(&r2).contains("two"));
        assert!(get_out(&r3).contains("three"));

        handle.close_async().await?;
        server_task.abort();
        Ok(())
    }
}
