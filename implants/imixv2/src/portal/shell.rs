use anyhow::Result;
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use tokio::sync::mpsc;

pub async fn handle_shell(
    first_mote: Mote,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    sequencer: PayloadSequencer,
    // Channel to send data TO the Shell (Input)
    shell_input_tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {

    // Process first mote
    process_shell_mote(first_mote, &out_tx, &sequencer, &shell_input_tx).await?;

    // Loop
    while let Some(mote) = rx.recv().await {
        process_shell_mote(mote, &out_tx, &sequencer, &shell_input_tx).await?;
    }

    Ok(())
}

async fn process_shell_mote(
    mote: Mote,
    out_tx: &mpsc::Sender<Mote>,
    sequencer: &PayloadSequencer,
    shell_input_tx: &mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    // Handle Ping (BytesPayload) or ShellPayload
    match mote.payload {
        Some(Payload::Bytes(b)) => {
             if b.kind == BytesPayloadKind::Ping as i32 {
                 // Echo Ping
                 let resp = sequencer.new_bytes_mote(b.data, BytesPayloadKind::Ping);
                 out_tx.send(resp).await.map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
             }
        },
        Some(Payload::Shell(s)) => {
            // Send data to Shell
             shell_input_tx.send(s.data).await.map_err(|e| anyhow::anyhow!("Shell input send failed: {}", e))?;
        },
        _ => {}
    }
    Ok(())
}
