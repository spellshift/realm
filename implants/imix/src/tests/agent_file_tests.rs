use crate::agent::ImixAgent;
use eldritch::Interpreter;
use pb::config::Config;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use transport::MockTransport;

fn create_dummy_file(size_mb: usize) -> (PathBuf, File) {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    path.push(format!("test_file_{}.dat", nanos));
    let file = File::create(&path).expect("Failed to create temp file");

    file.set_len((size_mb * 1024 * 1024) as u64)
        .expect("Failed to set len");
    file.sync_all().expect("Failed to sync");

    (path, file)
}

#[tokio::test]
async fn test_report_large_file_via_eldritch() {
    let _ = pretty_env_logger::try_init();

    // 1. Create 100MB file
    let (path, _file) = create_dummy_file(100);
    let path_str = path.to_string_lossy().to_string();

    // 2. Setup MockTransport
    let mut transport = MockTransport::default();

    // Expect clone (ImixAgent clones transport)
    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| true);

        t.expect_report_file().withf(|_rx| true).returning(|rx| {
            let mut chunk_count = 0;
            let mut total_bytes = 0;
            let mut first = true;

            for msg in rx {
                chunk_count += 1;
                if let Some(chunk) = msg.chunk {
                    total_bytes += chunk.chunk.len();
                    if first {
                        assert!(chunk.metadata.is_some(), "First chunk must have metadata");
                        assert!(msg.context.is_some(), "First chunk must have context");
                        first = false;
                    } else {
                        assert!(
                            chunk.metadata.is_none(),
                            "Subsequent chunks must NOT have metadata"
                        );
                        assert!(
                            msg.context.is_none(),
                            "Subsequent chunks must NOT have context"
                        );
                    }
                }
            }

            println!(
                "Received {} chunks, total {} bytes",
                chunk_count, total_bytes
            );
            assert!(
                chunk_count >= 100,
                "Should have at least 100 chunks for 100MB file"
            );
            assert_eq!(
                total_bytes,
                100 * 1024 * 1024,
                "Total bytes should match file size"
            );

            Ok(pb::c2::ReportFileResponse {})
        });

        t
    });

    transport.expect_is_active().returning(|| true);

    // 3. Create ImixAgent
    let handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(crate::task::TaskRegistry::new());
    let config = Config::default();

    let agent = Arc::new(ImixAgent::new(config, transport, handle, task_registry));

    // 4. Create Interpreter
    let mut interpreter = Interpreter::new().with_agent(agent);

    // 5. Run script
    let escaped_path = path_str.replace("\\", "\\\\");
    let script = format!("report.file(\"{}\")", escaped_path);

    println!("Running script: {}", script);

    // Run in a separate thread to allow block_on to work (since we are in tokio runtime here)
    let result = std::thread::spawn(move || interpreter.interpret(&script))
        .join()
        .expect("Thread panicked");

    // Cleanup
    let _ = std::fs::remove_file(path);

    assert!(
        result.is_ok(),
        "Script execution failed: {:?}",
        result.err()
    );
}
