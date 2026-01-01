use criterion::{BenchmarkId, Criterion};
use pb::c2::ReportFileRequest;
use pb::eldritch::{File, FileMetadata};
use prost::Message;

use crate::fixtures;

/// Benchmark protobuf serialization overhead
pub fn bench_protobuf_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage3_serialization");

    let chunk_sizes = vec![
        1024,              // 1 KB
        256 * 1024,        // 256 KB
        1024 * 1024,       // 1 MB
        2 * 1024 * 1024,   // 2 MB
    ];

    for chunk_size in chunk_sizes {
        group.throughput(criterion::Throughput::Bytes(chunk_size as u64));

        // Benchmark encoding ReportFileRequest to protobuf
        group.bench_with_input(
            BenchmarkId::new("encode_request", fixtures::human_readable(chunk_size)),
            &chunk_size,
            |b, &size| {
                let chunk_data = vec![0xEF; size];

                b.iter_batched(
                    || ReportFileRequest {
                        task_id: 1,
                        chunk: Some(File {
                            metadata: Some(FileMetadata {
                                path: "/tmp/test.bin".to_string(),
                                ..Default::default()
                            }),
                            chunk: chunk_data.clone(),
                        }),
                    },
                    |req| {
                        let encoded = req.encode_to_vec();
                        std::hint::black_box(encoded);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Benchmark serialization overhead (encoded size vs raw chunk size)
        group.bench_with_input(
            BenchmarkId::new("serialization_overhead", fixtures::human_readable(chunk_size)),
            &chunk_size,
            |b, &size| {
                let chunk_data = vec![0xEF; size];

                b.iter_batched(
                    || ReportFileRequest {
                        task_id: 1,
                        chunk: Some(File {
                            metadata: Some(FileMetadata {
                                path: "/tmp/test.bin".to_string(),
                                ..Default::default()
                            }),
                            chunk: chunk_data.clone(),
                        }),
                    },
                    |req| {
                        let encoded = req.encode_to_vec();
                        let overhead = encoded.len() - size;
                        std::hint::black_box(overhead); // Protobuf overhead
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark encoding with and without metadata
pub fn bench_metadata_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage3_metadata");

    let chunk_size = 1024 * 1024; // 1 MB
    let chunk_data = vec![0xEF; chunk_size];

    // With full metadata
    group.bench_function("with_metadata", |b| {
        b.iter_batched(
            || ReportFileRequest {
                task_id: 1,
                chunk: Some(File {
                    metadata: Some(FileMetadata {
                        path: "/tmp/test.bin".to_string(),
                        owner: "root".to_string(),
                        group: "root".to_string(),
                        permissions: "0644".to_string(),
                        size: chunk_size as u64,
                        sha3_256_hash: "abc123".to_string(),
                    }),
                    chunk: chunk_data.clone(),
                }),
            },
            |req| {
                let encoded = req.encode_to_vec();
                std::hint::black_box(encoded);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Without metadata (subsequent chunks)
    group.bench_function("without_metadata", |b| {
        b.iter_batched(
            || ReportFileRequest {
                task_id: 1,
                chunk: Some(File {
                    metadata: None,
                    chunk: chunk_data.clone(),
                }),
            },
            |req| {
                let encoded = req.encode_to_vec();
                std::hint::black_box(encoded);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
