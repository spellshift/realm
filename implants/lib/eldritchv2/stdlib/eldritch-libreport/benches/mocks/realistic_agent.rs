use eldritch_agent::Agent;
use pb::c2::*;
use prost::Message;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Metrics collected during benchmarking
#[derive(Debug, Clone)]
pub struct TransportMetrics {
    pub total_bytes: usize,
    pub encrypted_bytes: usize,
    pub chunks: usize,
}

/// Realistic agent that simulates network behavior for benchmarking
///
/// This agent:
/// - Actually performs encryption (to measure real overhead)
/// - Simulates network latency
/// - Simulates bandwidth limits
/// - Tracks detailed metrics
pub struct RealisticAgent {
    total_bytes: Arc<AtomicUsize>,
    encrypted_bytes: Arc<AtomicUsize>,
    chunks_received: Arc<AtomicUsize>,
    latency_per_message: Duration,
    bandwidth_limit: Option<usize>,
}

impl RealisticAgent {
    pub fn new(latency: Duration, bandwidth: Option<usize>) -> Self {
        Self {
            total_bytes: Arc::new(AtomicUsize::new(0)),
            encrypted_bytes: Arc::new(AtomicUsize::new(0)),
            chunks_received: Arc::new(AtomicUsize::new(0)),
            latency_per_message: latency,
            bandwidth_limit: bandwidth,
        }
    }

    /// Create a fast local network configuration
    pub fn fast_network() -> Self {
        Self::new(
            Duration::from_micros(100),           // 0.1ms latency
            Some(1000 * 1024 * 1024),            // 1 GB/s bandwidth
        )
    }

    /// Create a typical network configuration
    pub fn typical_network() -> Self {
        Self::new(
            Duration::from_millis(5),             // 5ms latency
            Some(10 * 1024 * 1024),              // 10 MB/s bandwidth
        )
    }

    /// Create a slow network configuration
    pub fn slow_network() -> Self {
        Self::new(
            Duration::from_millis(20),            // 20ms latency
            Some(1 * 1024 * 1024),               // 1 MB/s bandwidth
        )
    }

    /// Get metrics collected during benchmarking
    pub fn metrics(&self) -> TransportMetrics {
        TransportMetrics {
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            encrypted_bytes: self.encrypted_bytes.load(Ordering::Relaxed),
            chunks: self.chunks_received.load(Ordering::Relaxed),
        }
    }

    /// Reset metrics for a new benchmark run
    pub fn reset_metrics(&self) {
        self.total_bytes.store(0, Ordering::Relaxed);
        self.encrypted_bytes.store(0, Ordering::Relaxed);
        self.chunks_received.store(0, Ordering::Relaxed);
    }
}

impl Agent for RealisticAgent {
    fn report_file(&self, req: ReportFileRequest) -> Result<ReportFileResponse, String> {
        // Simulate network latency
        std::thread::sleep(self.latency_per_message);

        // Serialize and encrypt (like real transport)
        let plaintext = req.encode_to_vec();
        let ciphertext = pb::xchacha::encode_with_chacha::<
            ReportFileRequest,
            ReportFileResponse,
        >(req.clone())
            .map_err(|e| e.to_string())?;

        // Track metrics
        self.total_bytes.fetch_add(plaintext.len(), Ordering::Relaxed);
        self.encrypted_bytes.fetch_add(ciphertext.len(), Ordering::Relaxed);
        self.chunks_received.fetch_add(1, Ordering::Relaxed);

        // Simulate bandwidth throttling (simplified)
        if let Some(bw_limit) = self.bandwidth_limit {
            let transfer_time = Duration::from_secs_f64(
                ciphertext.len() as f64 / bw_limit as f64
            );
            if transfer_time > self.latency_per_message {
                std::thread::sleep(transfer_time - self.latency_per_message);
            }
        }

        Ok(ReportFileResponse::default())
    }

    // Implement all other required Agent trait methods as no-ops

    fn fetch_asset(&self, _req: FetchAssetRequest) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    fn report_credential(
        &self,
        _req: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse, String> {
        Ok(ReportCredentialResponse::default())
    }

    fn report_process_list(
        &self,
        _req: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse, String> {
        Ok(ReportProcessListResponse::default())
    }

    fn report_task_output(
        &self,
        _req: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse, String> {
        Ok(ReportTaskOutputResponse::default())
    }

    fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }

    fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }

    fn claim_tasks(&self, _req: ClaimTasksRequest) -> Result<ClaimTasksResponse, String> {
        Ok(ClaimTasksResponse::default())
    }

    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn get_transport(&self) -> Result<String, String> {
        Ok("realistic_mock".to_string())
    }

    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(vec!["realistic_mock".to_string()])
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(1000)
    }

    fn set_callback_interval(&self, _interval: u64) -> Result<(), String> {
        Ok(())
    }

    fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
        Ok(BTreeSet::new())
    }

    fn get_active_callback_uri(&self) -> Result<String, String> {
        Ok("http://localhost:8080".to_string())
    }

    fn get_next_callback_uri(&self) -> Result<String, String> {
        Ok("http://localhost:8080".to_string())
    }

    fn add_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn remove_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn list_tasks(&self) -> Result<Vec<pb::c2::Task>, String> {
        Ok(Vec::new())
    }

    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }
}
