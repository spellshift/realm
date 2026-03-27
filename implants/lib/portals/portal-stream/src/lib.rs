pub mod reader;
pub mod sequencer;
pub mod writer;

pub use reader::OrderedReader;
pub use sequencer::PayloadSequencer;
pub use writer::OrderedWriter;

#[cfg(feature = "tokio")]
use pb::portal::Mote;

#[cfg(feature = "tokio")]
impl OrderedWriter<tokio::sync::mpsc::Sender<Mote>> {
    /// Helper to create a new OrderedWriter from a tokio Sender.
    pub fn new_tokio(
        stream_id: impl Into<String>,
        sender: tokio::sync::mpsc::Sender<Mote>,
    ) -> Self {
        Self::new(stream_id, sender)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pb::portal::{BytesPayloadKind, mote::Payload};
    use std::thread;
    use std::time::Duration;

    fn make_mote(seq_id: u64) -> Mote {
        Mote {
            stream_id: "test".to_string(),
            seq_id,
            payload: None,
        }
    }

    #[test]
    fn test_sequencer() {
        let seq = PayloadSequencer::new("test");
        assert_eq!(seq.next_seq_id(), 0);
        assert_eq!(seq.next_seq_id(), 1);
        assert_eq!(seq.next_seq_id(), 2);

        let mote = seq.new_bytes_mote(vec![1, 2, 3], BytesPayloadKind::Data);
        assert_eq!(mote.seq_id, 3);
        assert_eq!(mote.stream_id, "test");
        if let Some(Payload::Bytes(b)) = mote.payload {
            assert_eq!(b.data, vec![1, 2, 3]);
        } else {
            panic!("expected bytes payload");
        }
    }

    #[test]
    fn test_reader_sequential() {
        let mut reader = OrderedReader::new();

        // 0
        let res = reader.process(make_mote(0)).unwrap();
        assert!(res.is_some());
        let motes = res.unwrap();
        assert_eq!(motes.len(), 1);
        assert_eq!(motes[0].seq_id, 0);

        // 1
        let res = reader.process(make_mote(1)).unwrap();
        assert!(res.is_some());
        let motes = res.unwrap();
        assert_eq!(motes.len(), 1);
        assert_eq!(motes[0].seq_id, 1);
    }

    #[test]
    fn test_reader_out_of_order() {
        let mut reader = OrderedReader::new();

        // 2 (gap)
        let res = reader.process(make_mote(2)).unwrap();
        assert!(res.is_none());

        // 0
        let res = reader.process(make_mote(0)).unwrap();
        assert!(res.is_some());
        let motes = res.unwrap();
        assert_eq!(motes.len(), 1);
        assert_eq!(motes[0].seq_id, 0);

        // 1 (fills gap to 2)
        let res = reader.process(make_mote(1)).unwrap();
        assert!(res.is_some());
        let motes = res.unwrap();
        assert_eq!(motes.len(), 2);
        assert_eq!(motes[0].seq_id, 1);
        assert_eq!(motes[1].seq_id, 2);
    }

    #[test]
    fn test_reader_duplicates() {
        let mut reader = OrderedReader::new();

        reader.process(make_mote(0)).unwrap();

        // 0 again
        let res = reader.process(make_mote(0)).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_reader_timeout() {
        let mut reader = OrderedReader::new().with_stale_buffer_timeout(Duration::from_millis(100));

        // 1 (gap)
        reader.process(make_mote(1)).unwrap();

        thread::sleep(Duration::from_millis(200));

        // Processing another packet (even if gap or duplicate) should trigger timeout check
        let err = reader.process(make_mote(2));
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err().to_string(),
            "stale stream: timeout waiting for seqID 0"
        );
    }

    #[test]
    fn test_reader_check_timeout() {
        let mut reader = OrderedReader::new().with_stale_buffer_timeout(Duration::from_millis(100));

        // 1 (gap)
        reader.process(make_mote(1)).unwrap();

        thread::sleep(Duration::from_millis(200));

        let err = reader.check_timeout();
        assert!(err.is_err());
    }

    #[test]
    fn test_reader_buffer_limit() {
        let mut reader = OrderedReader::new().with_max_buffered_messages(2);

        reader.process(make_mote(2)).unwrap();
        reader.process(make_mote(3)).unwrap();

        let err = reader.process(make_mote(4));
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err().to_string(),
            "stale stream: buffer limit exceeded"
        );
    }

    #[test]
    fn test_writer() {
        let mut output = Vec::new();
        let writer_func = |mote: Mote| {
            output.push(mote);
            Ok(())
        };

        let mut writer = OrderedWriter::new("test", writer_func);
        writer.write_bytes(vec![1], BytesPayloadKind::Data).unwrap();
        writer.write_bytes(vec![2], BytesPayloadKind::Data).unwrap();

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].seq_id, 0);
        assert_eq!(output[1].seq_id, 1);
    }

    #[test]
    fn test_writer_sync() {
        let mut output = Vec::new();
        let writer_func = |mote: Mote| {
            output.push(mote);
            Ok(())
        };

        let mut writer = OrderedWriter::new("test", writer_func);
        writer
            .write_bytes(vec![1, 2], BytesPayloadKind::Data)
            .unwrap();
        writer
            .write_tcp(vec![3, 4], "127.0.0.1".to_string(), 80)
            .unwrap();
        writer
            .write_udp(vec![5, 6], "127.0.0.1".to_string(), 53)
            .unwrap();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0].seq_id, 0);
        assert_eq!(output[1].seq_id, 1);
        assert_eq!(output[2].seq_id, 2);

        if let Some(Payload::Tcp(t)) = &output[1].payload {
            assert_eq!(t.data, vec![3, 4]);
            assert_eq!(t.dst_addr, "127.0.0.1");
            assert_eq!(t.dst_port, 80);
        } else {
            panic!("expected tcp payload");
        }

        if let Some(Payload::Udp(u)) = &output[2].payload {
            assert_eq!(u.data, vec![5, 6]);
            assert_eq!(u.dst_addr, "127.0.0.1");
            assert_eq!(u.dst_port, 53);
        } else {
            panic!("expected udp payload");
        }
    }

    #[test]
    fn test_writer_sync_error() {
        let writer_func = |_mote: Mote| -> Result<(), String> { Err("sync error".to_string()) };

        let mut writer = OrderedWriter::new("test", writer_func);
        let res1 = writer.write_bytes(vec![1, 2], BytesPayloadKind::Data);
        assert!(res1.is_err());
        assert_eq!(res1.unwrap_err(), "sync error");

        let res2 = writer.write_tcp(vec![3, 4], "127.0.0.1".to_string(), 80);
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err(), "sync error");

        let res3 = writer.write_udp(vec![5, 6], "127.0.0.1".to_string(), 53);
        assert!(res3.is_err());
        assert_eq!(res3.unwrap_err(), "sync error");
    }
}

#[cfg(feature = "tokio")]
#[cfg(test)]
mod tokio_tests {
    use super::*;
    use pb::portal::{BytesPayloadKind, mote::Payload};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_writer_tokio_async() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut writer = OrderedWriter::new_tokio("test_async", tx);

        writer
            .write_bytes_async(vec![1, 2], BytesPayloadKind::Data)
            .await
            .unwrap();
        writer
            .write_tcp_async(vec![3, 4], "127.0.0.1".to_string(), 80)
            .await
            .unwrap();
        writer
            .write_udp_async(vec![5, 6], "127.0.0.1".to_string(), 53)
            .await
            .unwrap();

        let m1 = rx.recv().await.unwrap();
        let m2 = rx.recv().await.unwrap();
        let m3 = rx.recv().await.unwrap();

        assert_eq!(m1.seq_id, 0);
        assert_eq!(m2.seq_id, 1);
        assert_eq!(m3.seq_id, 2);

        if let Some(Payload::Bytes(b)) = m1.payload {
            assert_eq!(b.data, vec![1, 2]);
        } else {
            panic!("expected bytes payload");
        }

        if let Some(Payload::Tcp(t)) = m2.payload {
            assert_eq!(t.data, vec![3, 4]);
            assert_eq!(t.dst_addr, "127.0.0.1");
            assert_eq!(t.dst_port, 80);
        } else {
            panic!("expected tcp payload");
        }

        if let Some(Payload::Udp(u)) = m3.payload {
            assert_eq!(u.data, vec![5, 6]);
            assert_eq!(u.dst_addr, "127.0.0.1");
            assert_eq!(u.dst_port, 53);
        } else {
            panic!("expected udp payload");
        }
    }

    #[tokio::test]
    async fn test_writer_tokio_async_error() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx); // Close the channel

        let mut writer = OrderedWriter::new_tokio("test_async", tx);

        let err1 = writer
            .write_bytes_async(vec![1, 2], BytesPayloadKind::Data)
            .await;
        assert!(err1.is_err());

        let err2 = writer
            .write_tcp_async(vec![3, 4], "127.0.0.1".to_string(), 80)
            .await;
        assert!(err2.is_err());

        let err3 = writer
            .write_udp_async(vec![5, 6], "127.0.0.1".to_string(), 53)
            .await;
        assert!(err3.is_err());
    }
}
