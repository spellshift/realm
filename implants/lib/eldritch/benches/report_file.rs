use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use eldritch::runtime::messages::{AsyncDispatcher, ReportFileMessage};
use pb::c2::{ReportFileRequest, ReportFileResponse};
use pb::config::Config;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use transport::MockTransport;

/// Creates a temporary file with the specified size filled with pseudo-random data.
/// The data pattern is deterministic (based on index) for reproducibility.
fn create_test_file(size_bytes: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let data: Vec<u8> = (0..size_bytes).map(|i| (i % 256) as u8).collect();
    file.write_all(&data).expect("Failed to write test data");
    file.flush().expect("Failed to flush file");
    file
}

/// Sets up a MockTransport that consumes all chunks from the receiver.
/// This simulates the transport layer receiving and processing file chunks.
fn setup_mock_transport() -> MockTransport {
    let mut transport = MockTransport::default();

    transport
        .expect_report_file()
        .times(1)
        .returning(|receiver: Receiver<ReportFileRequest>| {
            // Consume all chunks from the receiver to simulate transport processing
            while let Ok(_req) = receiver.recv() {
                // Just consume the chunks without tracking
                // In a real implementation, we would process the chunks
            }

            // Return success after consuming all chunks
            Ok(ReportFileResponse {})
        });

    transport
}

/// Formats file sizes in a human-readable format (B, KB, MB).
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}MB", bytes / (1024 * 1024))
    }
}

/// Benchmark report_file performance across different file sizes.
/// Tests ranging from 1KB (single chunk) to 10MB (10,240 chunks).
///
/// This measures end-to-end timing including:
/// - File I/O (reading from disk)
/// - Chunking (breaking file into 1KB pieces)
/// - Channel communication (sync_channel with backpressure)
/// - Mock transport consumption (simulating network transmission)
fn bench_report_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("report_file_by_size");

    // Test a range of file sizes to understand scaling characteristics
    let file_sizes = [
        1024,             // 1 KB - Single chunk baseline
        10 * 1024,        // 10 KB - Small multi-chunk
        100 * 1024,       // 100 KB - Medium multi-chunk
        1024 * 1024,      // 1 MB - Large file
        5 * 1024 * 1024,  // 5 MB - Very large file
        10 * 1024 * 1024, // 10 MB - Maximum tested size
    ];

    for &size in &file_sizes {
        // Set throughput to automatically calculate MB/s
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format_size(size)),
            &size,
            |b, &size| {
                // Use iter_custom to have precise control over timing
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        // Create a new runtime for each iteration to ensure isolation
                        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

                        let duration = rt.block_on(async {
                            // Create test file
                            let file = create_test_file(size);
                            let path = file.path().to_str().unwrap().to_string();

                            // Setup mock transport
                            let mut transport = setup_mock_transport();

                            // Create the message
                            let msg = ReportFileMessage::new(1, path.clone());

                            // Measure end-to-end time
                            let start = Instant::now();
                            msg.dispatch(&mut transport, Config::default())
                                .await
                                .expect("Failed to dispatch message");
                            start.elapsed()
                        });

                        total_duration += duration;
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_report_file_sizes);
criterion_main!(benches);
