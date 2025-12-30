pub mod reader;
pub mod sequencer;
pub mod writer;

pub use reader::{OrderedReader, ReaderError};
pub use sequencer::PayloadSequencer;
pub use writer::OrderedWriter;

#[cfg(test)]
mod tests {
    use super::*;
    use pb::portal::{mote::Payload, BytesPayloadKind, Mote};
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
        match err.unwrap_err() {
            ReaderError::Timeout(seq) => assert_eq!(seq, 0),
            _ => panic!("expected timeout error"),
        }
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
        match err.unwrap_err() {
            ReaderError::BufferLimitExceeded => {}
            _ => panic!("expected buffer limit error"),
        }
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
}
