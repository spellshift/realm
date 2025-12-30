use anyhow::{Result, Context};
use pb::portal::{mote::Payload, Mote};
use portal_stream::PayloadSequencer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

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

    let stream = TcpStream::connect(&addr).await.context("Failed to connect TCP")?;
    let (mut read_half, mut write_half) = tokio::io::split(stream);

    // If initial data exists, write it
    if !initial_data.is_empty() {
         write_half.write_all(&initial_data).await?;
    }

    // Spawn reader task (Socket -> C2)
    // We pass `sequencer` to the read task as it generates outgoing motes.
    let out_tx_clone = out_tx.clone();
    let dst_addr_clone = dst_addr.clone();

    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match read_half.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = buf[0..n].to_vec();
                    let mote = sequencer.new_tcp_mote(data, dst_addr_clone.clone(), dst_port);
                    if out_tx_clone.send(mote).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Write Loop (C2 -> Socket)
    while let Some(mote) = rx.recv().await {
        if let Some(Payload::Tcp(tcp)) = mote.payload {
            if !tcp.data.is_empty() {
                if write_half.write_all(&tcp.data).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = read_task.await;

    Ok(())
}
