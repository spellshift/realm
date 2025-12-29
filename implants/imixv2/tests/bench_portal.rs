use anyhow::Result;
use imixv2::portal::run_create_portal;
use pb::c2::{beacon, *};
use pb::portal::payload::Payload as PortalPayloadEnum;
use pb::portal::TcpMessage;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};
use transport::Transport;

// 1. Define BenchTransport
#[derive(Clone)]
struct BenchTransport {
    // We use Arc<Mutex<Option<...>>> to take the channels once inside create_portal
    c2_in_rx: Arc<Mutex<Option<mpsc::Receiver<CreatePortalResponse>>>>,
    c2_out_tx: Arc<mpsc::Sender<CreatePortalRequest>>,
}

impl Transport for BenchTransport {
    fn init() -> Self {
        unimplemented!()
    }
    fn new(_: String, _: Option<String>) -> Result<Self> {
        unimplemented!()
    }
    async fn claim_tasks(&mut self, _: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        unimplemented!()
    }
    async fn fetch_asset(
        &mut self,
        _: FetchAssetRequest,
        _: std::sync::mpsc::Sender<FetchAssetResponse>,
    ) -> Result<()> {
        unimplemented!()
    }
    async fn report_credential(
        &mut self,
        _: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        unimplemented!()
    }
    async fn report_file(
        &mut self,
        _: std::sync::mpsc::Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        unimplemented!()
    }
    async fn report_process_list(
        &mut self,
        _: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        unimplemented!()
    }
    async fn report_task_output(
        &mut self,
        _: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        unimplemented!()
    }
    async fn reverse_shell(
        &mut self,
        _: mpsc::Receiver<ReverseShellRequest>,
        _: mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn create_portal(
        &mut self,
        mut rx: mpsc::Receiver<CreatePortalRequest>, // Imix -> C2 (read from this)
        tx: mpsc::Sender<CreatePortalResponse>,      // C2 -> Imix (write to this)
    ) -> Result<()> {
        // Bridge C2 -> Imix
        let mut c2_in = self.c2_in_rx.lock().await.take().unwrap();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = c2_in.recv().await {
                if tx_clone.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Bridge Imix -> C2
        while let Some(msg) = rx.recv().await {
            if self.c2_out_tx.send(msg).await.is_err() {
                break;
            }
        }
        Ok(())
    }

    fn get_type(&mut self) -> beacon::Transport {
        beacon::Transport::Unspecified
    }
    fn is_active(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "bench"
    }
    fn list_available(&self) -> Vec<String> {
        vec!["bench".to_string()]
    }
}

#[tokio::test]
async fn benchmark_portal_throughput() {
    // 1. Setup Mock HTTP Server (1GB)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server in background
    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    // Read request (consume it)
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;

                    // Send 1GB of data
                    // We send in chunks to simulate streaming
                    let chunk_size = 64 * 1024; // 64KB
                    let total_size = 1024 * 1024 * 1024; // 1GB
                    let mut sent = 0;
                    let data = vec![b'x'; chunk_size]; // Pre-allocate chunk

                    while sent < total_size {
                        let remaining = total_size - sent;
                        let to_send = std::cmp::min(remaining, chunk_size);
                        if socket.write_all(&data[..to_send]).await.is_err() {
                            break;
                        }
                        sent += to_send;
                    }
                });
            }
        }
    });

    // 2. Setup Transport Channels
    let (c2_in_tx, c2_in_rx) = mpsc::channel(100); // Test -> Imix
    let (c2_out_tx, mut c2_out_rx) = mpsc::channel(100); // Imix -> Test

    let transport = BenchTransport {
        c2_in_rx: Arc::new(Mutex::new(Some(c2_in_rx))),
        c2_out_tx: Arc::new(c2_out_tx),
    };

    // 3. Start Imix
    let task_id = 999;
    tokio::spawn(async move {
        let _ = run_create_portal(task_id, transport).await;
    });

    // 4. Wait for Imix to be ready (it sends an initial request)
    let _init_req = c2_out_rx.recv().await.unwrap();

    // 5. Send Trigger (TCP Message) to Imix
    println!("Starting benchmark: Downloading 1GB via portal...");
    let start_time = Instant::now();

    c2_in_tx.send(CreatePortalResponse {
        payload: Some(pb::portal::Payload {
            seq_id: 1,
            payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                src_id: "bench_conn".to_string(),
                dst_addr: "127.0.0.1".to_string(),
                dst_port: addr.port() as u32,
                data: b"GET / HTTP/1.1\r\n\r\n".to_vec(),
            })),
        }),
    }).await.unwrap();

    // 6. Receive Data
    let mut total_received = 0;
    let expected_size = 1024 * 1024 * 1024;

    while let Some(msg) = c2_out_rx.recv().await {
        if let Some(payload) = msg.payload.and_then(|p| p.payload) {
            match payload {
                PortalPayloadEnum::Tcp(tcp) => {
                    total_received += tcp.data.len();
                    if total_received >= expected_size {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let duration = start_time.elapsed();
    let seconds = duration.as_secs_f64();
    let mb = total_received as f64 / (1024.0 * 1024.0);
    let throughput = mb / seconds;

    println!("Benchmark Complete!");
    println!("Total Received: {} bytes", total_received);
    println!("Time: {:.2} seconds", seconds);
    println!("Throughput: {:.2} MB/s", throughput);
}
