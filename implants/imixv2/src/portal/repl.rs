use anyhow::Result;
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use tokio::sync::mpsc;

pub async fn handle_repl(
    first_mote: Mote,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    sequencer: PayloadSequencer,
    task_id: i64,
    repl_input_tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {

    // Process first mote
    process_repl_mote(first_mote, &out_tx, &sequencer, &repl_input_tx).await?;

    // Loop
    while let Some(mote) = rx.recv().await {
        process_repl_mote(mote, &out_tx, &sequencer, &repl_input_tx).await?;
    }

    Ok(())
}

async fn process_repl_mote(
    mote: Mote,
    out_tx: &mpsc::Sender<Mote>,
    sequencer: &PayloadSequencer,
    repl_input_tx: &mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    // Handle Ping (BytesPayload) or ReplPayload
    match mote.payload {
        Some(Payload::Bytes(b)) => {
             if b.kind == BytesPayloadKind::Ping as i32 {
                 // Echo Ping
                 let resp = sequencer.new_bytes_mote(b.data, BytesPayloadKind::Ping);
                 out_tx.send(resp).await.map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
             }
             // Ignore other bytes? Or treat as data?
             // Legacy treated everything as data unless Ping.
        },
        Some(Payload::Repl(r)) => {
            // Send data to REPL
             repl_input_tx.send(r.data).await.map_err(|e| anyhow::anyhow!("REPL input send failed: {}", e))?;
        },
        _ => {}
    }
    Ok(())
}
