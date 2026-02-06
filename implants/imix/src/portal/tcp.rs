use anyhow::{Context, Result};
use pb::portal::{Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

const BUF_SIZE: usize = 256 * 1024;
const BATCH_DURATION: Duration = Duration::from_millis(1);
const MAX_BATCH_SIZE: usize = 2097152;

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

    #[cfg(debug_assertions)]
    log::debug!("Connecting TCP to {}", addr);

    let stream = TcpStream::connect(&addr)
        .await
        .context("Failed to connect TCP")?;

    // Disable Nagle's algorithm
    stream
        .set_nodelay(true)
        .context("Failed to set TCP_NODELAY")?;

    #[cfg(debug_assertions)]
    log::info!(
        "Connected TCP to {} (local: {:?})",
        addr,
        stream.local_addr()
    );

    let (mut read_half, mut write_half) = stream.into_split();

    // If initial data exists, write it
    if !initial_data.is_empty() {
        write_half.write_all(&initial_data).await?;
    }

    // Spawn reader task (Socket -> C2)
    // We pass `sequencer` to the read task as it generates outgoing motes.
    let out_tx_clone = out_tx.clone();
    let dst_addr_clone = dst_addr.clone();

    #[cfg(debug_assertions)]
    let addr_for_read = addr.clone();

    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; BUF_SIZE];
        loop {
            // Wait for the first read (blocking)
            match read_half.read(&mut buf).await {
                Ok(0) => {
                    #[cfg(debug_assertions)]
                    log::info!("TCP connection closed by remote peer: {}", addr_for_read);
                    break; // EOF
                }
                Ok(n) => {
                    let mut batch = buf[0..n].to_vec();

                    // Collect additional data within the batch window
                    let deadline = sleep(BATCH_DURATION);
                    tokio::pin!(deadline);
                    loop {
                        if batch.len() >= MAX_BATCH_SIZE {
                            break;
                        }
                        tokio::select! {
                            biased;
                            result = read_half.read(&mut buf) => {
                                match result {
                                    Ok(0) => {
                                        // EOF — flush what we have, then exit
                                        break;
                                    }
                                    Ok(n) => {
                                        batch.extend_from_slice(&buf[0..n]);
                                    }
                                    Err(_e) => {
                                        #[cfg(debug_assertions)]
                                        log::error!("Error reading from TCP socket {}: {:?}", addr_for_read, _e);
                                        break;
                                    }
                                }
                            }
                            () = &mut deadline => {
                                break;
                            }
                        }
                    }

                    #[cfg(debug_assertions)]
                    log::debug!(
                        "← TCP {} {} bytes from portal stream (batched)",
                        dst_addr_clone.clone(),
                        batch.len()
                    );

                    let mote =
                        sequencer.new_tcp_mote(batch, dst_addr_clone.clone(), dst_port);
                    if out_tx_clone.send(mote).await.is_err() {
                        #[cfg(debug_assertions)]
                        log::warn!(
                            "Failed to send mote to C2 (channel closed) for {}",
                            addr_for_read
                        );
                        break;
                    }
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("Error reading from TCP socket {}: {:?}", addr_for_read, _e);
                    break;
                }
            }
        }
    });

    // Write Loop (C2 -> Socket)
    'outer: while let Some(mote) = rx.recv().await {
        // Collect the first mote's data
        let mut batch = Vec::new();
        if let Some(Payload::Tcp(tcp)) = mote.payload {
            if !tcp.data.is_empty() {
                batch.extend_from_slice(&tcp.data);
            }
        }

        // Collect additional motes within the batch window
        let deadline = sleep(BATCH_DURATION);
        tokio::pin!(deadline);
        loop {
            if batch.len() >= MAX_BATCH_SIZE {
                break;
            }
            tokio::select! {
                biased;
                maybe_mote = rx.recv() => {
                    match maybe_mote {
                        Some(m) => {
                            if let Some(Payload::Tcp(tcp)) = m.payload {
                                if !tcp.data.is_empty() {
                                    batch.extend_from_slice(&tcp.data);
                                }
                            }
                        }
                        None => break, // channel closed
                    }
                }
                () = &mut deadline => {
                    break;
                }
            }
        }

        if !batch.is_empty() {
            #[cfg(debug_assertions)]
            let n = batch.len();

            match write_half.write_all(&batch).await {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::debug!("→ TCP {dst_addr} {n} bytes to portal stream (batched)");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to write tcp ({n} bytes) to {}: {_e:?}", addr);

                    break 'outer;
                }
            }
        }
    }

    // Cleanup
    let _ = write_half.shutdown().await;
    let _ = read_task.await;

    Ok(())
}
