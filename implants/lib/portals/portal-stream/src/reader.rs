use anyhow::{Result, anyhow};
use pb::portal::Mote;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// OrderedReader receives Motes and ensures they are read in order.
pub struct OrderedReader {
    next_seq_id: u64,
    buffer: BTreeMap<u64, Mote>,
    max_buffer: usize,
    stale_timeout: Duration,
    first_buffered_at: Option<Instant>,
}

impl OrderedReader {
    /// Creates a new OrderedReader with default settings.
    pub fn new() -> Self {
        Self {
            next_seq_id: 0,
            buffer: BTreeMap::new(),
            max_buffer: 1024,
            stale_timeout: Duration::from_secs(5),
            first_buffered_at: None,
        }
    }

    /// Sets the maximum number of out-of-order messages to buffer.
    pub fn with_max_buffered_messages(mut self, max: usize) -> Self {
        self.max_buffer = max;
        self
    }

    /// Sets the duration to wait for the next expected sequence ID before erroring.
    pub fn with_stale_buffer_timeout(mut self, timeout: Duration) -> Self {
        self.stale_timeout = timeout;
        self
    }

    /// Processes a new mote.
    /// Returns:
    /// - Ok(Some(vec![...])): A list of ordered motes ready to be consumed.
    /// - Ok(None): The mote was buffered (gap detected) or dropped (duplicate).
    /// - Err(e): Buffer limit exceeded or stale timeout detected *during* processing.
    pub fn process(&mut self, mote: Mote) -> Result<Option<Vec<Mote>>> {
        // Check for duplicate/old packet
        if mote.seq_id < self.next_seq_id {
            return Ok(None);
        }

        // Check if this is the expected packet
        if mote.seq_id == self.next_seq_id {
            let mut result = Vec::new();
            result.push(mote);
            self.next_seq_id += 1;

            // Check if we have subsequent packets in the buffer
            while let Some(next_mote) = self.buffer.remove(&self.next_seq_id) {
                result.push(next_mote);
                self.next_seq_id += 1;
            }

            // Update timer based on buffer state
            if self.buffer.is_empty() {
                self.first_buffered_at = None;
            } else {
                // Buffer still has items (gaps further ahead), so we are technically waiting for the next one.
                // Resetting timer to give full timeout for the next expected packet.
                self.first_buffered_at = Some(Instant::now());
            }

            return Ok(Some(result));
        }

        // Gap detected: mote.seq_id > self.next_seq_id
        if self.buffer.is_empty() {
            self.first_buffered_at = Some(Instant::now());
        }

        self.buffer.entry(mote.seq_id).or_insert(mote);

        if self.buffer.len() > self.max_buffer {
            return Err(anyhow!("stale stream: buffer limit exceeded"));
        }

        // We check timeout here as well, similar to Go implementation
        self.check_timeout()?;

        Ok(None)
    }

    /// Checks if the reader has stalled waiting for a packet.
    pub fn check_timeout(&self) -> Result<()> {
        if !self.buffer.is_empty()
            && let Some(start) = self.first_buffered_at
            && start.elapsed() > self.stale_timeout
        {
            return Err(anyhow!(
                "stale stream: timeout waiting for seqID {}",
                self.next_seq_id
            ));
        }
        Ok(())
    }
}

impl Default for OrderedReader {
    fn default() -> Self {
        Self::new()
    }
}
