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
    use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
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

        let tcp_mote = seq.new_tcp_mote(vec![4, 5, 6], "127.0.0.1".to_string(), 8080);
        assert_eq!(tcp_mote.seq_id, 4);
        assert_eq!(tcp_mote.stream_id, "test");
        if let Some(Payload::Tcp(t)) = tcp_mote.payload {
            assert_eq!(t.data, vec![4, 5, 6]);
            assert_eq!(t.dst_addr, "127.0.0.1");
            assert_eq!(t.dst_port, 8080);
        } else {
            panic!("expected tcp payload");
        }

        let udp_mote = seq.new_udp_mote(vec![7, 8, 9], "127.0.0.1".to_string(), 53);
        assert_eq!(udp_mote.seq_id, 5);
        assert_eq!(udp_mote.stream_id, "test");
        if let Some(Payload::Udp(u)) = udp_mote.payload {
            assert_eq!(u.data, vec![7, 8, 9]);
            assert_eq!(u.dst_addr, "127.0.0.1");
            assert_eq!(u.dst_port, 53);
        } else {
            panic!("expected udp payload");
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
        use std::sync::{Arc, Mutex};
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();

        let writer_func = move |mote: Mote| {
            output_clone.lock().unwrap().push(mote);
            Ok(())
        };

        let mut writer = OrderedWriter::new("test", writer_func);
        writer.write_bytes(vec![1], BytesPayloadKind::Data).unwrap();
        writer.write_bytes(vec![2], BytesPayloadKind::Data).unwrap();

        {
            let out = output.lock().unwrap();
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].seq_id, 0);
            assert_eq!(out[1].seq_id, 1);
        }

        writer
            .write_tcp(vec![3], "127.0.0.1".to_string(), 80)
            .unwrap();
        {
            let out = output.lock().unwrap();
            assert_eq!(out.len(), 3);
            assert_eq!(out[2].seq_id, 2);
            if let Some(Payload::Tcp(t)) = &out[2].payload {
                assert_eq!(t.data, vec![3]);
                assert_eq!(t.dst_addr, "127.0.0.1");
                assert_eq!(t.dst_port, 80);
            } else {
                panic!("expected tcp payload");
            }
        }

        writer
            .write_udp(vec![4], "127.0.0.1".to_string(), 53)
            .unwrap();
        {
            let out = output.lock().unwrap();
            assert_eq!(out.len(), 4);
            assert_eq!(out[3].seq_id, 3);
            if let Some(Payload::Udp(u)) = &out[3].payload {
                assert_eq!(u.data, vec![4]);
                assert_eq!(u.dst_addr, "127.0.0.1");
                assert_eq!(u.dst_port, 53);
            } else {
                panic!("expected udp payload");
            }
        }
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_writer_async() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let mut writer = OrderedWriter::new_tokio("test", tx);

        writer
            .write_bytes_async(vec![1], BytesPayloadKind::Data)
            .await
            .unwrap();
        writer
            .write_tcp_async(vec![2], "127.0.0.1".to_string(), 80)
            .await
            .unwrap();
        writer
            .write_udp_async(vec![3], "127.0.0.1".to_string(), 53)
            .await
            .unwrap();

        let mote1 = rx.recv().await.unwrap();
        assert_eq!(mote1.seq_id, 0);
        if let Some(Payload::Bytes(b)) = mote1.payload {
            assert_eq!(b.data, vec![1]);
        } else {
            panic!("expected bytes payload");
        }

        let mote2 = rx.recv().await.unwrap();
        assert_eq!(mote2.seq_id, 1);
        if let Some(Payload::Tcp(t)) = mote2.payload {
            assert_eq!(t.data, vec![2]);
        } else {
            panic!("expected tcp payload");
        }

        let mote3 = rx.recv().await.unwrap();
        assert_eq!(mote3.seq_id, 2);
        if let Some(Payload::Udp(u)) = mote3.payload {
            assert_eq!(u.data, vec![3]);
        } else {
            panic!("expected udp payload");
        }
    }
}
