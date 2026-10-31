use anyhow::{Context, Result};
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

const BUF_SIZE: usize = 64 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn handle_tcp(
    first_mote: Mote,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    sequencer: PayloadSequencer,
) -> Result<()> {
    // Extract destination from first mote
    let (dst_addr, dst_port, initial_data) = if let Some(Payload::Tcp(tcp)) = first_mote.payload {
        (tcp.dst_addr, tcp.dst_port, tcp.data)
    } else {
        return Err(anyhow::anyhow!("Expected TCP payload"));
    };

    let addr = format!("{}:{}", dst_addr, dst_port);

    #[cfg(feature = "print_debug")]
    log::debug!("Connecting TCP to {}", addr);

    let stream = match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr)).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            let err_msg = format!("Failed to connect TCP to {}: {}", addr, e);
            let close_mote =
                sequencer.new_bytes_mote(err_msg.clone().into_bytes(), BytesPayloadKind::Close);
            let _ = out_tx.send(close_mote).await;
            return Err(anyhow::anyhow!(err_msg));
        }
        Err(_) => {
            let err_msg = format!("TCP connection to {} timed out", addr);
            let close_mote =
                sequencer.new_bytes_mote(err_msg.clone().into_bytes(), BytesPayloadKind::Close);
            let _ = out_tx.send(close_mote).await;
            return Err(anyhow::anyhow!(err_msg));
        }
    };

    // Disable Nagle's algorithm
    stream
        .set_nodelay(true)
        .context("Failed to set TCP_NODELAY")?;

    #[cfg(feature = "print_debug")]
    log::info!(
        "Connected TCP to {} (local: {:?})",
        addr,
        stream.local_addr()
    );

    let (mut read_half, mut write_half) = stream.into_split();

    // If initial data exists, write it
    if !initial_data.is_empty()
        && let Err(e) = write_half.write_all(&initial_data).await
    {
        let err_msg = format!("Failed to write initial TCP data to {}: {}", addr, e);
        let close_mote =
            sequencer.new_bytes_mote(err_msg.clone().into_bytes(), BytesPayloadKind::Close);
        let _ = out_tx.send(close_mote).await;
        return Err(anyhow::anyhow!(err_msg));
    }

    // Spawn reader task (Socket -> C2)
    // We move `sequencer` to the read task as it generates outgoing motes.
    let out_tx_clone = out_tx.clone();
    let dst_addr_clone = dst_addr.clone();

    #[cfg(feature = "print_debug")]
    let addr_for_read = addr.clone();

    let mut read_task = tokio::spawn(async move {
        let mut buf = [0u8; BUF_SIZE];
        loop {
            match read_half.read(&mut buf).await {
                Ok(0) => {
                    #[cfg(feature = "print_debug")]
                    log::info!("TCP connection closed by remote peer: {}", addr_for_read);
                    let close_mote =
                        sequencer.new_bytes_mote(b"EOF".to_vec(), BytesPayloadKind::Close);
                    let _ = out_tx_clone.send(close_mote).await;
                    break; // EOF
                }
                Ok(n) => {
                    #[cfg(feature = "print_debug")]
                    log::debug!(
                        "← TCP {} {n} bytes from portal stream ",
                        dst_addr_clone.clone()
                    );

                    let data = buf[0..n].to_vec();
                    let mote = sequencer.new_tcp_mote(data, dst_addr_clone.clone(), dst_port);
                    if out_tx_clone.send(mote).await.is_err() {
                        #[cfg(feature = "print_debug")]
                        log::warn!(
                            "Failed to send mote to C2 (channel closed) for {}",
                            addr_for_read
                        );
                        break;
                    }
                }
                Err(e) => {
                    #[cfg(feature = "print_debug")]
                    log::error!("Error reading from TCP socket {}: {:?}", addr_for_read, e);
                    let close_mote = sequencer
                        .new_bytes_mote(e.to_string().into_bytes(), BytesPayloadKind::Close);
                    let _ = out_tx_clone.send(close_mote).await;
                    break;
                }
            }
        }
    });

    // Write Loop (C2 -> Socket) or exit when remote side closes
    loop {
        tokio::select! {
            _ = &mut read_task => {
                break;
            }
            maybe_mote = rx.recv() => {
                let Some(mote) = maybe_mote else {
                    read_task.abort();
                    break;
                };
                match mote.payload {
                    Some(Payload::Tcp(tcp)) if !tcp.data.is_empty() => {
                        #[cfg(feature = "print_debug")]
                        let n = tcp.data.len();

                        match write_half.write_all(&tcp.data).await {
                            Ok(_) => {
                                #[cfg(feature = "print_debug")]
                                log::debug!("→ TCP {dst_addr} {n} bytes to portal stream ");
                            }
                            Err(_e) => {
                                #[cfg(feature = "print_debug")]
                                log::error!("failed to write tcp ({n} bytes)to {}: {_e:?}", addr);
                                read_task.abort();
                                break;
                            }
                        }
                    }
                    Some(Payload::Bytes(b)) if b.kind == BytesPayloadKind::Close as i32 => {
                        read_task.abort();
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup
    let _ = write_half.shutdown().await;

    Ok(())
}
