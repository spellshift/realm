use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use eldritch::runtime::messages::{AsyncDispatcher, ReportFileMessage};
use pb::c2::{ReportFileRequest, ReportFileResponse};
use pb::config::Config;
use pb::xchacha::{decode_with_chacha, encode_with_chacha};
use prost::Message;
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
/// This simulates the transport layer receiving and processing file chunks WITHOUT crypto.
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

/// Sets up a MockTransport that encrypts/decrypts each chunk with ChaCha20-Poly1305.
/// This simulates the REAL transport overhead including crypto codec processing.
fn setup_mock_transport_with_codec() -> MockTransport {
    let mut transport = MockTransport::default();

    transport
        .expect_report_file()
        .times(1)
        .returning(|receiver: Receiver<ReportFileRequest>| {
            // Process chunks with encryption/decryption to simulate real codec overhead
            while let Ok(req) = receiver.recv() {
                // Encode the request to protobuf bytes
                let proto_bytes = req.encode_to_vec();

                // Encrypt the chunk (simulating ChachaCodec.encode)
                let encrypted = encode_with_chacha::<ReportFileRequest, ReportFileResponse>(req)
                    .expect("Encryption failed");

                // Decrypt the chunk (simulating ChachaCodec.decode on server side)
                let _decrypted: ReportFileRequest =
                    decode_with_chacha::<ReportFileResponse, ReportFileRequest>(&encrypted)
                        .expect("Decryption failed");

                // Use black_box to prevent compiler optimization
                black_box(_decrypted);
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

/// Benchmark comparing report_file performance WITH and WITHOUT crypto codec.
/// This directly measures the 3x latency overhead caused by ChaCha20-Poly1305 encryption.
///
/// For each file size, two benchmarks are run:
/// - "no_codec": MockTransport without encryption (baseline)
/// - "with_codec": MockTransport with ChaCha20-Poly1305 encryption/decryption per chunk
fn bench_report_file_codec_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("report_file_codec_comparison");

    // Focus on medium file sizes where codec overhead is most apparent
    let file_sizes = [
        10 * 1024,       // 10 KB - 10 chunks
        100 * 1024,      // 100 KB - 100 chunks
        1024 * 1024,     // 1 MB - 1024 chunks
        5 * 1024 * 1024, // 5 MB - 5120 chunks
    ];

    for &size in &file_sizes {
        group.throughput(Throughput::Bytes(size as u64));

        // Benchmark WITHOUT codec (baseline)
        group.bench_with_input(
            BenchmarkId::new("no_codec", format_size(size)),
            &size,
            |b, &size| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;
                    for _ in 0..iters {
                        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                        let duration = rt.block_on(async {
                            let file = create_test_file(size);
                            let path = file.path().to_str().unwrap().to_string();
                            let mut transport = setup_mock_transport();
                            let msg = ReportFileMessage::new(1, path.clone());

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

        // Benchmark WITH codec (real-world scenario)
        group.bench_with_input(
            BenchmarkId::new("with_codec", format_size(size)),
            &size,
            |b, &size| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;
                    for _ in 0..iters {
                        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                        let duration = rt.block_on(async {
                            let file = create_test_file(size);
                            let path = file.path().to_str().unwrap().to_string();
                            let mut transport = setup_mock_transport_with_codec();
                            let msg = ReportFileMessage::new(1, path.clone());

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

/// Benchmark just the crypto overhead per chunk (isolated test).
/// This measures ONLY the encryption/decryption time for a single 1KB chunk.
fn bench_codec_per_chunk(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec_per_chunk");

    // Create a sample 1KB chunk
    let chunk_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let sample_request = ReportFileRequest {
        task_id: 1,
        chunk: Some(pb::eldritch::File {
            metadata: Some(pb::eldritch::FileMetadata {
                path: "/tmp/test".to_string(),
                owner: String::new(),
                group: String::new(),
                permissions: String::new(),
                size: 0,
                sha3_256_hash: String::new(),
            }),
            chunk: chunk_data,
        }),
    };

    // Benchmark encryption only
    group.bench_function("encrypt_1KB", |b| {
        b.iter(|| {
            let req = sample_request.clone();
            let encrypted = encode_with_chacha::<ReportFileRequest, ReportFileResponse>(req)
                .expect("Encryption failed");
            black_box(encrypted);
        });
    });

    // Benchmark full encrypt + decrypt cycle
    group.bench_function("encrypt_decrypt_1KB", |b| {
        b.iter(|| {
            let req = sample_request.clone();
            let encrypted = encode_with_chacha::<ReportFileRequest, ReportFileResponse>(req)
                .expect("Encryption failed");
            let _decrypted: ReportFileRequest =
                decode_with_chacha::<ReportFileResponse, ReportFileRequest>(&encrypted)
                    .expect("Decryption failed");
            black_box(_decrypted);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_report_file_sizes,
    bench_report_file_codec_comparison,
    bench_codec_per_chunk
);
criterion_main!(benches);
