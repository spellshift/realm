use alloc::sync::Arc;
use core::pin::Pin;
use core::task::{Context, Poll};

use js_sys::Function;

use russh::keys::PublicKey;
use russh::ChannelId;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;
use wasm_bindgen::prelude::*;
use log::debug;

/// Wraps JS callbacks to implement custom Read/Write streams.
pub struct WasmTcpStream {
    recv_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    on_send: js_sys::Function,
    buffer: Vec<u8>,
}

impl WasmTcpStream {
    pub fn new(recv_rx: mpsc::UnboundedReceiver<Vec<u8>>, on_send: js_sys::Function) -> Self {
        Self {
            recv_rx,
            on_send,
            buffer: Vec::new(),
        }
    }
}

impl AsyncRead for WasmTcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if !self.buffer.is_empty() {
            let space = buf.remaining();
            let to_read = core::cmp::min(space, self.buffer.len());
            buf.put_slice(&self.buffer[..to_read]);
            self.buffer.drain(..to_read);
            return Poll::Ready(Ok(()));
        }

        match self.recv_rx.poll_recv(cx) {
            Poll::Ready(Some(mut data)) => {
                let space = buf.remaining();
                let to_read = core::cmp::min(space, data.len());
                buf.put_slice(&data[..to_read]);
                
                if data.len() > to_read {
                    data.drain(..to_read);
                    self.buffer.extend(data);
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(None) => Poll::Ready(Ok(())), // EOF
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for WasmTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let js_array = js_sys::Uint8Array::from(buf);
        let _ = self.on_send.call1(&JsValue::NULL, &JsValue::from(js_array));
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

struct SshHandler {
    on_stdout: Function,
    on_stderr: Function,
    on_disconnect: Function,
}

impl russh::client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true) // Accept all keys for browser terminal usage
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        let js_array = js_sys::Uint8Array::from(data);
        let _ = self.on_stdout.call1(&JsValue::NULL, &JsValue::from(js_array));
        Ok(())
    }

    async fn extended_data(
        &mut self,
        _channel: ChannelId,
        _ext: u32,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        let js_array = js_sys::Uint8Array::from(data);
        let _ = self.on_stderr.call1(&JsValue::NULL, &JsValue::from(js_array));
        Ok(())
    }

    async fn disconnected(
        &mut self,
        reason: russh::client::DisconnectReason<Self::Error>,
    ) -> Result<(), Self::Error> {
        let msg = format!("SSH disconnected: {:?}", reason);
        let _ = self.on_disconnect.call1(&JsValue::NULL, &JsValue::from_str(&msg));
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WasmSsh {
    tcp_recv_tx: mpsc::UnboundedSender<Vec<u8>>,
    stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    resize_tx: mpsc::UnboundedSender<(u32, u32)>,
}

#[wasm_bindgen]
impl WasmSsh {
    #[wasm_bindgen(constructor)]
    pub fn new(
        target_user: String,
        on_send: Function,
        on_stdout: Function,
        on_stderr: Function,
        on_disconnect: Function,
        cols: u32,
        rows: u32,
    ) -> WasmSsh {
        let (tcp_recv_tx, tcp_recv_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u32, u32)>();

        wasm_bindgen_futures::spawn_local(async move {
            let config = Arc::new(russh::client::Config::default());
            let handler = SshHandler {
                on_stdout,
                on_stderr,
                on_disconnect: on_disconnect.clone(),
            };

            let stream = WasmTcpStream::new(tcp_recv_rx, on_send);

            web_sys::console::log_1(&"[WasmSsh] connecting stream...".into());

            // Build a JS-based 15-second timeout that works without a tokio time driver.
            // tokio::time::timeout panics in WASM because wasm_bindgen_futures doesn't
            // set up the tokio time subsystem.
            let timeout_promise = js_sys::Promise::new(&mut |resolve, _reject| {
                let window = web_sys::window().expect("no global window");
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 15_000)
                    .expect("set_timeout failed");
            });
            let timeout_fut = wasm_bindgen_futures::JsFuture::from(timeout_promise);

            let connect_fut = russh::client::connect_stream(config, stream, handler);

            // Race the SSH handshake against the 15-second JS timeout.
            let connect_res;
            futures::pin_mut!(connect_fut, timeout_fut);
            match futures::future::select(connect_fut, timeout_fut).await {
                futures::future::Either::Left((res, _timeout)) => {
                    connect_res = res;
                }
                futures::future::Either::Right((_js_val, _connect)) => {
                    let msg = "Connection timed out — is sshd running on the target, and is port 22 reachable?";
                    web_sys::console::log_1(&format!("[WasmSsh] {}", msg).into());
                    let _ = on_disconnect.call1(&JsValue::NULL, &JsValue::from_str(msg));
                    return;
                }
            }

            match connect_res {
                Ok(mut session) => {
                    web_sys::console::log_1(&"[WasmSsh] SSH handshake complete, authenticating...".into());
                    let auth_res = session.authenticate_none(target_user).await;

                    let authed = match auth_res {
                        Ok(russh::client::AuthResult::Success) => {
                            web_sys::console::log_1(&"[WasmSsh] none-auth succeeded".into());
                            true
                        }
                        Ok(other) => {
                            web_sys::console::log_1(
                                &format!("[WasmSsh] none-auth not accepted: {:?} — server may require password/key", other).into()
                            );
                            false
                        }
                        Err(e) => {
                            web_sys::console::log_1(
                                &format!("[WasmSsh] authenticate_none error: {:?}", e).into()
                            );
                            false
                        }
                    };

                    if !authed {
                        let _ = on_disconnect.call1(
                            &JsValue::NULL,
                            &JsValue::from_str("Authentication failed (none-auth rejected — server may require a password or key)"),
                        );
                        return;
                    }

                    web_sys::console::log_1(&"[WasmSsh] opening session channel...".into());
                    match session.channel_open_session().await {
                        Ok(mut channel) => {
                            web_sys::console::log_1(&"[WasmSsh] channel open, requesting PTY + shell".into());
                            let _ = channel.request_pty(true, "xterm-256color", cols, rows, 0, 0, &[]).await;
                            let _ = channel.request_shell(true).await;
                            web_sys::console::log_1(&"[WasmSsh] shell active, entering I/O loop".into());

                            loop {
                                tokio::select! {
                                    Some(data) = stdin_rx.recv() => {
                                        let _ = channel.data(&data[..]).await;
                                    }
                                    Some((c, r)) = resize_rx.recv() => {
                                        let _ = channel.window_change(c, r, 0, 0).await;
                                    }
                                    Some(msg) = channel.wait() => {
                                        match msg {
                                            russh::ChannelMsg::Eof => break,
                                            russh::ChannelMsg::Close => break,
                                            _ => {}
                                        }
                                    }
                                    else => break,
                                }
                            }
                            web_sys::console::log_1(&"[WasmSsh] I/O loop exited".into());
                        }
                        Err(e) => {
                            let msg = format!("Failed to open SSH channel: {:?}", e);
                            web_sys::console::log_1(&msg.clone().into());
                            let _ = on_disconnect.call1(&JsValue::NULL, &JsValue::from_str(&msg));
                            return;
                        }
                    }
                }
                Err(e) => {
                    // Produce a user-readable message — common case is connection refused
                    let msg = if format!("{:?}", e).contains("ConnectionRefused") || format!("{}", e).contains("Connection refused") {
                        format!("Connection refused — is SSH running on the target?")
                    } else {
                        format!("SSH connection failed: {}", e)
                    };
                    web_sys::console::log_1(&format!("[WasmSsh] {}", msg).into());
                    let _ = on_disconnect.call1(&JsValue::NULL, &JsValue::from_str(&msg));
                    return;
                }
            }
            let _ = on_disconnect.call1(&JsValue::NULL, &JsValue::from_str("SSH session ended"));
        });

        WasmSsh {
            tcp_recv_tx,
            stdin_tx,
            resize_tx,
        }
    }

    pub fn on_tcp_recv(&self, data: &[u8]) {
        let _ = self.tcp_recv_tx.send(data.to_vec());
    }

    pub fn on_stdin(&self, data: &[u8]) {
        let _ = self.stdin_tx.send(data.to_vec());
    }

    pub fn resize_pty(&self, cols: u32, rows: u32) {
        let _ = self.resize_tx.send((cols, rows));
    }
}
